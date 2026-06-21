fn main() {
    let output = "<|channel>thought\nThinking Process:\n\n1.  **Analyze the Request:**".to_string();
    let start_tag = "".to_string();
    let end_tag = "".to_string();
    
    let mut thinking = String::new();
    let mut answer = output.clone();

    if let Some(start_idx) = output.find(&start_tag) {
        if let Some(end_idx) = output.find(&end_tag) {
            let think_content = &output[start_idx + start_tag.len()..end_idx];
            thinking = think_content.trim().to_string();
            answer = output[end_idx + end_tag.len()..].trim().to_string();
        } else {
            thinking = output[start_idx + start_tag.len()..].trim().to_string();
            answer = String::new();
        }
    }
    
    println!("thinking: {:?}", thinking);
    println!("answer: {:?}", answer);
}
