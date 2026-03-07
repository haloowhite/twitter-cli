use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;
use regex::Regex;
use rquest::Client;
use rquest_util::Emulation;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::f64::consts::PI;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

static ON_DEMAND_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"['"]ondemand\.s['"]\s*:\s*['"](\w*)['"]"#).unwrap()
});

static INDICES_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\(\w\[(\d{1,2})\],\s*16\)"#).unwrap()
});

const HASH_KEYWORD: &str = "obfiowerehiring";
const EPOCH_OFFSET_MS: u64 = 1682924400 * 1000;
const TOTAL_ANIMATION_TIME: f64 = 4096.0;
const RANDOM_SUFFIX: u8 = 3;
const CACHE_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Serialize, Deserialize)]
struct TransactionCache {
    key_bytes: Vec<u8>,
    animation_key: String,
    created_at: u64,
}

pub struct ClientTransaction {
    key_bytes: Vec<u8>,
    animation_key: String,
}

fn cache_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".x-cli")
        .join("transaction_cache.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl ClientTransaction {
    pub async fn new() -> Result<Self> {
        // Try loading from cache first
        if let Ok(cached) = Self::load_cache() {
            return Ok(cached);
        }

        // Fetch fresh data with retries
        let ct = Self::fetch_fresh().await?;
        ct.save_cache();
        Ok(ct)
    }

    fn load_cache() -> Result<Self> {
        let path = cache_path();
        let content = std::fs::read_to_string(&path)
            .context("No cache file")?;
        let cache: TransactionCache = serde_json::from_str(&content)
            .context("Invalid cache")?;

        if now_secs() - cache.created_at > CACHE_TTL_SECS {
            anyhow::bail!("Cache expired");
        }

        Ok(Self {
            key_bytes: cache.key_bytes,
            animation_key: cache.animation_key,
        })
    }

    fn save_cache(&self) {
        let cache = TransactionCache {
            key_bytes: self.key_bytes.clone(),
            animation_key: self.animation_key.clone(),
            created_at: now_secs(),
        };
        if let Ok(json) = serde_json::to_string(&cache) {
            let _ = std::fs::write(cache_path(), json);
        }
    }

    async fn fetch_fresh() -> Result<Self> {
        let client = Client::builder()
            .emulation(Emulation::Chrome136)
            .build()
            .context("Failed to build HTTP client for transaction")?;

        let mut last_err = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            match Self::fetch_once(&client).await {
                Ok(ct) => return Ok(ct),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap())
    }

    async fn fetch_once(client: &Client) -> Result<Self> {
        let home_html = client
            .get("https://x.com")
            .send()
            .await
            .context("Failed to fetch x.com homepage")?
            .text()
            .await
            .context("Failed to read homepage body")?;

        let document = scraper::Html::parse_document(&home_html);

        let js_hash = ON_DEMAND_FILE_REGEX
            .captures(&home_html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .context("Could not find ondemand.s file reference")?;

        let js_url = format!(
            "https://abs.twimg.com/responsive-web/client-web/ondemand.s.{js_hash}a.js"
        );
        let js_text = client
            .get(&js_url)
            .send()
            .await
            .context("Failed to fetch ondemand.s JS")?
            .text()
            .await?;

        let indices: Vec<usize> = INDICES_REGEX
            .captures_iter(&js_text)
            .filter_map(|c| c.get(1)?.as_str().parse().ok())
            .collect();

        if indices.is_empty() {
            anyhow::bail!("Could not extract key byte indices from JS");
        }

        let row_index = indices[0];
        let key_byte_indices: Vec<usize> = indices[1..].to_vec();

        let selector =
            scraper::Selector::parse("[name='twitter-site-verification']").unwrap();
        let key_b64 = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .context("Verification key not found in page")?;

        let key_bytes = STANDARD
            .decode(key_b64)
            .context("Failed to decode verification key")?;

        let animation_key =
            compute_animation_key(&document, &key_bytes, row_index, &key_byte_indices)?;

        Ok(Self {
            key_bytes,
            animation_key,
        })
    }

    pub fn generate(&self, method: &str, path: &str) -> String {
        let timestamp = get_timestamp();
        let timestamp_bytes = int_to_bytes(timestamp, 4);

        let hash_input = format!(
            "{method}!{path}!{timestamp}{HASH_KEYWORD}{}",
            self.animation_key
        );
        let hash: Vec<u8> = Sha256::digest(hash_input.as_bytes())[..16].to_vec();

        let mut payload = Vec::new();
        payload.extend_from_slice(&self.key_bytes);
        payload.extend_from_slice(&timestamp_bytes);
        payload.extend_from_slice(&hash);
        payload.push(RANDOM_SUFFIX);

        let xor_key: u8 = rand::rng().random();
        let mut encoded = vec![xor_key];
        for &b in &payload {
            encoded.push(b ^ xor_key);
        }

        let b64 = STANDARD.encode(&encoded);
        b64.trim_end_matches('=').to_string()
    }
}

fn get_timestamp() -> u32 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    ((now_ms - EPOCH_OFFSET_MS) / 1000) as u32
}

fn int_to_bytes(value: u32, num_bytes: usize) -> Vec<u8> {
    (0..num_bytes)
        .map(|i| ((value >> (i * 8)) & 0xFF) as u8)
        .collect()
}

fn compute_animation_key(
    document: &scraper::Html,
    key_bytes: &[u8],
    row_index: usize,
    key_byte_indices: &[usize],
) -> Result<String> {
    let row_idx = (key_bytes[row_index] as usize) % 16;
    let frame_time: f64 = key_byte_indices
        .iter()
        .map(|&i| (key_bytes[i] % 16) as f64)
        .product();
    let target_time = frame_time / TOTAL_ANIMATION_TIME;

    let frame_selector =
        scraper::Selector::parse("[id^='loading-x-anim']").unwrap();
    let frames: Vec<_> = document.select(&frame_selector).collect();

    if frames.is_empty() {
        anyhow::bail!("No animation frames found in page");
    }

    let frame_index = (key_bytes[5] as usize) % frames.len().max(1);
    let frame_el = frames
        .get(frame_index)
        .context("Frame index out of bounds")?;

    let path_selector = scraper::Selector::parse("path").unwrap();
    let paths: Vec<_> = frame_el.select(&path_selector).collect();

    let path_el = paths.get(1).or_else(|| paths.first()).context("No path element found")?;

    let d_attr = path_el
        .value()
        .attr("d")
        .context("No d attribute on path")?;

    let path_data = if d_attr.len() > 9 { &d_attr[9..] } else { d_attr };

    let frame_data: Vec<Vec<i32>> = path_data
        .split('C')
        .map(|segment| {
            let re = Regex::new(r"[^\d]+").unwrap();
            re.replace_all(segment.trim(), " ")
                .trim()
                .split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect()
        })
        .filter(|v: &Vec<i32>| !v.is_empty())
        .collect();

    if row_idx >= frame_data.len() {
        anyhow::bail!("Row index {row_idx} out of bounds for frame data");
    }

    let frame_row = &frame_data[row_idx];
    Ok(animate_frame(frame_row, target_time))
}

fn animate_frame(frame: &[i32], t: f64) -> String {
    if frame.len() < 11 {
        return String::new();
    }

    let from_color: Vec<f64> = vec![frame[0] as f64, frame[1] as f64, frame[2] as f64, 1.0];
    let to_color: Vec<f64> = vec![frame[3] as f64, frame[4] as f64, frame[5] as f64, 1.0];
    let to_rotation = scale_value(frame[6] as f64, 60.0, 360.0, true);

    let curve_data = &frame[7..];
    let curves: Vec<f64> = curve_data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let min = if i % 2 != 0 { -1.0 } else { 0.0 };
            scale_value(v as f64, min, 1.0, false)
        })
        .collect();

    let progress = if curves.len() >= 4 {
        cubic_bezier_evaluate(curves[0], curves[1], curves[2], curves[3], t)
    } else {
        t
    };

    let color = interpolate_lists(&from_color, &to_color, progress);
    let color: Vec<f64> = color.iter().map(|&v| v.max(0.0)).collect();

    let rotation = vec![0.0_f64.lerp(to_rotation, progress)];
    let matrix = rotation_to_matrix(rotation[0]);

    let mut hex_parts = Vec::new();
    for &v in &color[..3] {
        hex_parts.push(format!("{:x}", v.round() as i64));
    }
    for &v in &matrix {
        hex_parts.push(float_to_hex(v.abs().round_to(2)));
    }
    hex_parts.push("0".to_string());
    hex_parts.push("0".to_string());

    let joined = hex_parts.join("");
    joined.replace(['.', '-'], "")
}

fn scale_value(value: f64, min_val: f64, max_val: f64, floor: bool) -> f64 {
    let result = value * (max_val - min_val) / 255.0 + min_val;
    if floor {
        result.floor()
    } else {
        (result * 100.0).round() / 100.0
    }
}

fn rotation_to_matrix(degrees: f64) -> Vec<f64> {
    let radians = degrees * PI / 180.0;
    let cos_val = radians.cos();
    let sin_val = radians.sin();
    vec![cos_val, -sin_val, sin_val, cos_val]
}

fn interpolate_lists(from: &[f64], to: &[f64], t: f64) -> Vec<f64> {
    from.iter()
        .zip(to.iter())
        .map(|(&a, &b)| a * (1.0 - t) + b * t)
        .collect()
}

fn cubic_bezier_evaluate(x1: f64, y1: f64, x2: f64, y2: f64, target_x: f64) -> f64 {
    const EPSILON: f64 = 0.00001;

    if target_x <= 0.0 {
        if x1 > 0.0 {
            return (y1 / x1) * target_x;
        }
        return 0.0;
    }
    if target_x >= 1.0 {
        if x2 < 1.0 {
            return 1.0 + ((y2 - 1.0) / (x2 - 1.0)) * (target_x - 1.0);
        }
        return 1.0;
    }

    let mut low = 0.0_f64;
    let mut high = 1.0_f64;
    let mut mid;

    loop {
        mid = (low + high) / 2.0;
        let x_at_mid = bezier_component(x1, x2, mid);
        if (target_x - x_at_mid).abs() < EPSILON {
            break;
        }
        if x_at_mid < target_x {
            low = mid;
        } else {
            high = mid;
        }
        if (high - low) < EPSILON {
            break;
        }
    }

    bezier_component(y1, y2, mid)
}

fn bezier_component(p1: f64, p2: f64, t: f64) -> f64 {
    let omt = 1.0 - t;
    3.0 * p1 * omt * omt * t + 3.0 * p2 * omt * t * t + t * t * t
}

fn float_to_hex(value: f64) -> String {
    let value = value.abs();
    let int_part = value as u64;
    let frac_part = value - int_part as f64;

    let int_hex = if int_part == 0 {
        String::new()
    } else {
        format!("{:x}", int_part)
    };

    if frac_part == 0.0 || frac_part < 0.001 {
        if int_hex.is_empty() {
            "0".to_string()
        } else {
            int_hex.to_lowercase()
        }
    } else {
        let frac_hex = frac_to_hex(frac_part);
        let result = if int_hex.is_empty() {
            format!("0.{frac_hex}")
        } else {
            format!("{int_hex}.{frac_hex}")
        };
        result.to_lowercase()
    }
}

fn frac_to_hex(mut frac: f64) -> String {
    let mut digits = String::new();
    let mut count = 0;
    while frac > 0.0 && count < 8 {
        frac *= 16.0;
        let digit = frac as u64;
        frac -= digit as f64;
        digits.push_str(&format!("{:x}", digit));
        count += 1;
    }
    digits
}

trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
}

impl Lerp for f64 {
    fn lerp(self, other: f64, t: f64) -> f64 {
        self * (1.0 - t) + other * t
    }
}

trait RoundTo {
    fn round_to(self, decimals: u32) -> Self;
}

impl RoundTo for f64 {
    fn round_to(self, decimals: u32) -> f64 {
        let factor = 10_f64.powi(decimals as i32);
        (self * factor).round() / factor
    }
}
