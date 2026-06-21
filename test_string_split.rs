fn main() {
    let output = r#"<|channel>thought
Thinking Process:

1.  **Analyze the Request:** The user wants to know why the sky is blue. The constraint is to keep the explanation "very short."

2.  **Identify the Core Scientific Concept:** The phenomenon is Rayleigh scattering. Sunlight (white light) hits the Earth's atmosphere, and shorter wavelengths (blue/violet) are scattered more effectively than longer wavelengths (red/orange).

3.  **Determine the Steps for a Short Explanation:**
    *   Sunlight enters the atmosphere.
    *   The atmosphere scatters the light.
    *   Blue light is scattered most (Rayleigh scattering).
    *   This scattered blue light reaches our eyes.

4.  **Draft the Short Explanation (Iterative Refinement):**
    *   *Draft 1:* The sky is blue because sunlight hits the atmosphere and gets scattered by tiny molecules. Blue light scatters more than other colors.
    *   *Refinement for Clarity/Impact:* Focus on *why* blue scatters.

5.  **Final Polish (Ensure brevity):** Stick to the physics concept simply.

6.  **Generate Output.**<channel|>The sky is blue because of a phenomenon called Rayleigh scattering. Sunlight, which is made up of all colors, hits the Earth's atmosphere and is scattered by tiny gas molecules. Blue light is scattered most effectively, making the entire sky appear blue to our eyes."#;
    let mut effective_start = "<|channel>thought\n".to_string();
    let effective_end = "<channel|>".to_string();

    if !output.contains(&effective_start) {
        effective_start = String::new();
    }

    if !effective_start.is_empty() {
        if let Some(start_idx) = output.find(&effective_start) {
            if let Some(end_idx) = output.find(&effective_end) {
                let think_content = &output[start_idx + effective_start.len()..end_idx];
                let thinking = think_content.trim().to_string();
                let answer = output[end_idx + effective_end.len()..].trim().to_string();
                println!("SUCCESS!");
                println!("thinking length: {:?}", thinking.len());
                println!("answer prefix: {:?}", &answer[..20]);
            } else {
                println!("No end index");
            }
        } else {
            println!("No start index");
        }
    } else {
        println!("effective_start is empty");
    }
}
