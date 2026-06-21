fn main() {
    let output = "<|channel>thought\nThinking Process:\n\n1.  **Analyze the Request:** The user wants to know why the sky is blue. The response must be very short and follow a step-by-step explanation.\n\n2.  **Identify the Core Phenomenon:** The color of the sky is due to Rayleigh scattering.\n\n3.  **Determine the Steps for Explanation (Simplified):**\n    *   Step 1: Sunlight enters the atmosphere.\n    *   Step 2: The atmosphere scatters the light.\n    *   Step 3: Shorter wavelengths (blue/violet) are scattered more effectively than longer ones (red/orange).\n    *   Step 4: Our eyes perceive this scattered blue light.\n\n4.  **Draft the Short Explanation (Iterative Refinement):**\n    *   *Draft 1 (Too wordy):* The sun sends light to Earth. The atmosphere scatters this light. Blue light is scattered most effectively by the tiny molecules in the air, making the whole sky look blue to us.\n    *   *Draft 2 (Focusing on key concepts):* Sunlight hits the atmosphere. Air molecules scatter the light. Blue light is scattered more than other colors, making the sky appear blue.\n\n5.  **Final Review against Constraints:**\n    *   Think step by step? Yes (implicitly structured).\n    *   Explain why the sky is blue? Yes.\n    *   Keep it very short? Yes.\n\n6.  **Generate Output.** (This matches the desired concise explanation.)<channel|>1. **Sunlight enters the atmosphere.**\n2. **Air molecules scatter the light.**\n3. **Blue light is scattered most effectively**, making the entire sky appear blue to our eyes.".to_string();

    let start_tag = "<|channel>thought".to_string(); // Simulated DNA tag
    let end_tag = "</|channel>thought".to_string();

    let mut thinking = String::new();
    let mut answer = output.clone();

    let mut effective_start = start_tag.clone();
    let mut effective_end = end_tag.clone();

    if !effective_start.is_empty() && !output.contains(&effective_start) {
        effective_start = String::new();
        effective_end = String::new();
    }

    if effective_start.is_empty() {
        let pairs = [
            ("<think>", "</think>"),
            ("<thought>", "</thought>"),
            ("<|channel>thought\n", "<channel|>"),
            ("<|channel>thought", "<channel|>"),
            ("<|thought_start|>", "<|thought_end|>"),
            ("<|think|>", "</|think|>"),
        ];
        for (s, e) in pairs.iter() {
            if output.contains(s) {
                effective_start = s.to_string();
                effective_end = e.to_string();
                break;
            }
        }
    }

    if !effective_start.is_empty() {
        if let Some(start_idx) = output.find(&effective_start) {
            if let Some(end_idx) = output.find(&effective_end) {
                let think_content = &output[start_idx + effective_start.len()..end_idx];
                thinking = think_content.trim().to_string();
                answer = output[end_idx + effective_end.len()..].trim().to_string();
            } else {
                thinking = String::new();
                answer = output[start_idx + effective_start.len()..].trim().to_string();
            }
        }
    }

    println!("Effective Start: {:?}", effective_start);
    println!("Thinking is empty: {}", thinking.is_empty());
    println!("Answer: {:?}", answer);
}
