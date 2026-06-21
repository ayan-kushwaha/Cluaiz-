use std::fs;

fn main() {
    let template = fs::read_to_string("c:/Users/Aryan/.cluaize/models/chat/gemma4-e4b-gguf-q4_k_m/structural_dna.json").unwrap();
    let template_lower = template.to_lowercase();
    let mut detected_start = None;
    let mut detected_end = None;

    let keywords = [
        "think",
        "thought",
        "reasoning",
        "reason",
        "brainstorm",
        "logic",
    ];
    for kw in keywords.iter() {
        let formats = [
            (format!("<{}>", kw), format!("</{}>", kw)),
            (format!("<|{}_start|>", kw), format!("<|{}_end|>", kw)),
            (format!("<|{}|>", kw), format!("</|{}|>", kw)),
        ];

        for (start, end) in formats.iter() {
            if template_lower.contains(start) {
                detected_start = Some(start.clone());
                if template_lower.contains(end) {
                    detected_end = Some(end.clone());
                }
                break;
            }
        }
        if detected_start.is_some() {
            break;
        }
    }
    println!("Start: {:?}, End: {:?}", detected_start, detected_end);
}
