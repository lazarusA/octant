//! What arrives while a model loads, kept until it is ready.

use super::Input;

/// Samples of the audio heard while a model loads that are kept for it
/// (2 s): a quick load loses no first words, a long download transcribes
/// nothing stale.
const READY_TAIL: usize = super::SAMPLE_RATE as usize * 2;

/// What arrives while a model loads: the last [`READY_TAIL`] samples and
/// the stops among them, replayed in order once it is ready, so an
/// utterance stopped meanwhile still ends with its final text.
#[derive(Default)]
pub struct Tail {
    inputs: std::collections::VecDeque<Input>,
    samples: usize,
    /// The last prompt, given first.
    prompt: Option<String>,
}

impl Tail {
    pub fn push(&mut self, input: Input) {
        let input = match input {
            Input::Prompt(prompt) => return self.prompt = Some(prompt),
            input => input,
        };
        if let Input::Audio(samples) = &input {
            self.samples += samples.len();
        }
        self.inputs.push_back(input);
        while self.samples > READY_TAIL {
            let excess = self.samples - READY_TAIL;
            match self.inputs.front_mut() {
                Some(Input::Audio(samples)) if samples.len() > excess => {
                    samples.drain(..excess);
                    self.samples -= excess;
                }
                Some(Input::Audio(samples)) => {
                    self.samples -= samples.len();
                    self.inputs.pop_front();
                }
                Some(Input::Finish | Input::Prompt(_)) => {
                    self.inputs.pop_front();
                }
                None => break,
            }
        }
        // A stop with no audio before it ends nothing.
        while matches!(self.inputs.front(), Some(Input::Finish)) {
            self.inputs.pop_front();
        }
    }

    /// The inputs kept, the prompt first, then oldest first, leaving the
    /// tail empty.
    pub fn take(&mut self) -> impl Iterator<Item = Input> {
        self.samples = 0;
        let prompt = self.prompt.take().map(Input::Prompt);
        prompt.into_iter().chain(std::mem::take(&mut self.inputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_keeps_its_last_samples_and_their_stops() {
        let mut tail = Tail::default();
        tail.push(Input::Audio(vec![1.0; READY_TAIL]));
        tail.push(Input::Finish);
        tail.push(Input::Audio(vec![2.0; 10]));
        let inputs: Vec<Input> = tail.take().collect();
        let [Input::Audio(old), Input::Finish, Input::Audio(new)] = inputs.as_slice() else {
            panic!("audio, stop, audio");
        };
        assert_eq!((old.len(), new.len()), (READY_TAIL - 10, 10));
        // Once past the stop, the stop goes with the audio before it.
        tail.push(Input::Audio(vec![1.0; 10]));
        tail.push(Input::Finish);
        tail.push(Input::Audio(vec![2.0; READY_TAIL]));
        let inputs: Vec<Input> = tail.take().collect();
        assert!(matches!(inputs.as_slice(), [Input::Audio(a)] if a.len() == READY_TAIL));
    }
}
