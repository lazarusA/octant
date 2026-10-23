//! Whisper's files: OpenAI's own Hugging Face repositories, each pinned to a
//! commit and every file checked by SHA-256. The desktop downloads them into
//! its data folder (`download.rs`), the browser's voice worker into its
//! cache; Octant never hosts or ships them.

use crate::voice::stt::WhisperSize;

/// The weights' file name; the others are `config.json` and `tokenizer.json`.
pub const WEIGHTS: &str = "model.safetensors";

struct Spec {
    repo: &'static str,
    revision: &'static str,
    config: &'static str,
    tokenizer: &'static str,
    weights: &'static str,
    weights_bytes: u64,
}

/// `tokenizer.json` of every multilingual model before large-v3.
const TOKENIZER: &str = "27fc476bfe7f17299480be2273fc0608e4d5a99aba2ab5dec5374b4482d1a566";

fn spec(size: WhisperSize) -> Spec {
    match size {
        WhisperSize::Tiny => Spec {
            repo: "openai/whisper-tiny",
            revision: "169d4a4341b33bc18d8881c4b69c2e104e1cc0af",
            config: "ffdccec4f3211f4c63310f2b7098f309fe70f3952cedc5e4d11e43f5b2379b98",
            tokenizer: TOKENIZER,
            weights: "7ebd0e69e78190ffe1438491fa05cc1f5c1aa3a4c4db3bc1723adbb551ea2395",
            weights_bytes: 151_061_672,
        },
        WhisperSize::Base => Spec {
            repo: "openai/whisper-base",
            revision: "e37978b90ca9030d5170a5c07aadb050351a65bb",
            config: "a153c53883a6799b6f056b4a8d1a515c9926d03994682ba88a7616618d7da0c1",
            tokenizer: TOKENIZER,
            weights: "07cadb9f25677c8d50df603e66a98fbd842cce45047139baeb16e6219a1e807b",
            weights_bytes: 290_403_936,
        },
        WhisperSize::Small => Spec {
            repo: "openai/whisper-small",
            revision: "973afd24965f72e36ca33b3055d56a652f456b4d",
            config: "e6a2b489da1b5aed65a8eb8d1e7466fa867ad5643a8bc138ba708bd56b2875c4",
            tokenizer: TOKENIZER,
            weights: "1d7734884874f1a1513ed9aa760a4f8e97aaa02fd6d93a3a85d27b2ae9ca596b",
            weights_bytes: 966_995_080,
        },
        WhisperSize::Turbo => Spec {
            repo: "openai/whisper-large-v3-turbo",
            revision: "41f01f3fe87f28c78e2fbf8b568835947dd65ed9",
            config: "c5b526b3e3cd64cd8940dabb45e8ba726629e22d8ed389c29b552f9140daf04a",
            tokenizer: "297b13372ac43916285644fb9687add3cc62ee2a1adb60da3dc25cc94c1871fd",
            weights: "542566a422ae4f3fd23f1ba11add198fca01bbf82e66e6a2857b3f608b1eb9d1",
            weights_bytes: 1_617_824_864,
        },
    }
}

/// Megabytes the model's weights take.
pub fn megabytes(size: WhisperSize) -> u64 {
    weights_bytes(size) / 1_000_000
}

pub fn weights_bytes(size: WhisperSize) -> u64 {
    spec(size).weights_bytes
}

/// The model's files with their SHA-256, the weights last.
pub fn files(size: WhisperSize) -> [(&'static str, &'static str); 3] {
    let s = spec(size);
    [
        ("config.json", s.config),
        ("tokenizer.json", s.tokenizer),
        (WEIGHTS, s.weights),
    ]
}

/// Where `file` of the model is fetched from: its pinned revision.
pub fn url(size: WhisperSize, file: &str) -> String {
    let s = spec(size);
    format!(
        "https://huggingface.co/{}/resolve/{}/{file}",
        s.repo, s.revision
    )
}
