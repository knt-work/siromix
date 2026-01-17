// src-tauri/src/docx/mixer.rs
//! Exam mixing logic - shuffles questions and options to create exam variants
//! Ported from TypeScript for better performance with large documents

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::model::{OptionItem, Question, Segment};

/// A mixed exam variant with unique exam code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedExam {
    #[serde(rename = "examCode")]
    pub exam_code: String,
    pub questions: Vec<MixedQuestion>,
}

/// A question in a mixed exam (after shuffling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedQuestion {
    #[serde(rename = "originalNumber")]
    pub original_number: u32,
    #[serde(rename = "displayNumber")]
    pub display_number: u32,
    pub stem: Vec<Segment>,
    pub options: Vec<MixedOption>,
    #[serde(rename = "correctAnswer")]
    pub correct_answer: String,
}

/// An option after shuffling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedOption {
    pub label: String,
    #[serde(rename = "originalLabel")]
    pub original_label: String,
    pub content: Vec<Segment>,
}

/// Generate a random 3-digit exam code (100-999)
fn generate_exam_code(rng: &mut StdRng) -> String {
    use rand::Rng;
    rng.gen_range(100..=999).to_string()
}

/// Generate unique exam codes
fn generate_exam_codes(count: usize) -> Vec<String> {
    let mut codes = HashSet::new();
    let mut rng = StdRng::from_entropy();

    while codes.len() < count {
        codes.insert(generate_exam_code(&mut rng));
    }

    codes.into_iter().collect()
}

/// Shuffle options within a question and return mapping of old → new labels
fn shuffle_options(
    options: &[OptionItem],
    rng: &mut StdRng,
) -> (Vec<MixedOption>, HashMap<String, String>) {
    let labels = ["A", "B", "C", "D", "E", "F"];
    let mut shuffled = options.to_vec();
    shuffled.shuffle(rng);

    let mut mapping = HashMap::new();
    let mixed_options: Vec<MixedOption> = shuffled
        .iter()
        .enumerate()
        .map(|(idx, opt)| {
            let new_label = labels[idx].to_string();
            mapping.insert(opt.label.clone(), new_label.clone());

            MixedOption {
                label: new_label,
                original_label: opt.label.clone(),
                content: opt.content.clone(),
            }
        })
        .collect();

    (mixed_options, mapping)
}

/// Main mix function - creates multiple exam variants
///
/// # Arguments
/// * `questions` - Original parsed questions
/// * `num_variants` - Number of exam variants to generate
///
/// # Returns
/// Vector of MixedExam with shuffled questions and options
pub fn mix_exams(questions: Vec<Question>, num_variants: usize) -> Vec<MixedExam> {
    let mut variants = Vec::new();
    let exam_codes = generate_exam_codes(num_variants);

    for (variant_idx, exam_code) in exam_codes.iter().enumerate() {
        // Use different seed for each variant
        let seed = (variant_idx as u64).wrapping_mul(1000);
        let mut rng = StdRng::seed_from_u64(seed);

        // 1. Shuffle question order
        let mut shuffled_questions = questions.clone();
        shuffled_questions.shuffle(&mut rng);

        // 2. Process each question
        let mixed_questions: Vec<MixedQuestion> = shuffled_questions
            .iter()
            .enumerate()
            .map(|(idx, q)| {
                // Shuffle options with different seed for each question
                let question_seed = seed.wrapping_add(idx as u64);
                let mut question_rng = StdRng::seed_from_u64(question_seed);
                let (shuffled_options, mapping) = shuffle_options(&q.options, &mut question_rng);

                // Find new correct answer label
                let new_correct_label = mapping
                    .get(&q.correct_label)
                    .cloned()
                    .unwrap_or_else(|| q.correct_label.clone());

                MixedQuestion {
                    original_number: q.number,
                    display_number: (idx + 1) as u32,
                    stem: q.stem.clone(),
                    options: shuffled_options,
                    correct_answer: new_correct_label,
                }
            })
            .collect();

        variants.push(MixedExam {
            exam_code: exam_code.clone(),
            questions: mixed_questions,
        });
    }

    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_exam_codes() {
        let codes = generate_exam_codes(4);
        assert_eq!(codes.len(), 4);
        
        // All codes should be unique
        let unique: HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 4);

        // All codes should be 3 digits
        for code in &codes {
            let num: u32 = code.parse().unwrap();
            assert!(num >= 100 && num <= 999);
        }
    }

    #[test]
    fn test_mix_exams_generates_correct_count() {
        let questions = vec![
            Question {
                number: 1,
                stem: vec![Segment::Text {
                    text: "Question 1".to_string(),
                    raw_xml: String::new(),
                }],
                options: vec![
                    OptionItem {
                        label: "A".to_string(),
                        locked: false,
                        content: vec![Segment::Text {
                            text: "Option A".to_string(),
                            raw_xml: String::new(),
                        }],
                    },
                    OptionItem {
                        label: "B".to_string(),
                        locked: false,
                        content: vec![Segment::Text {
                            text: "Option B".to_string(),
                            raw_xml: String::new(),
                        }],
                    },
                ],
                correct_label: "A".to_string(),
            },
        ];

        let variants = mix_exams(questions, 3);
        assert_eq!(variants.len(), 3);
        
        // Each variant should have questions
        for variant in &variants {
            assert_eq!(variant.questions.len(), 1);
            assert!(!variant.exam_code.is_empty());
        }
    }

    #[test]
    fn test_shuffle_options_preserves_content() {
        let options = vec![
            OptionItem {
                label: "A".to_string(),
                locked: false,
                content: vec![Segment::Text {
                    text: "Option A".to_string(),
                    raw_xml: String::new(),
                }],
            },
            OptionItem {
                label: "B".to_string(),
                locked: false,
                content: vec![Segment::Text {
                    text: "Option B".to_string(),
                    raw_xml: String::new(),
                }],
            },
        ];

        let mut rng = StdRng::seed_from_u64(42);
        let (shuffled, mapping) = shuffle_options(&options, &mut rng);

        // Should have same number of options
        assert_eq!(shuffled.len(), 2);
        
        // Mapping should contain all original labels
        assert!(mapping.contains_key("A"));
        assert!(mapping.contains_key("B"));
        
        // New labels should be A and B (in some order)
        let new_labels: HashSet<_> = shuffled.iter().map(|o| o.label.as_str()).collect();
        assert!(new_labels.contains("A"));
        assert!(new_labels.contains("B"));
    }
}
