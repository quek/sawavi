use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    event::Event, model::note::Note, plugin::Plugin, process_track_context::ProcessTrackContext,
};

use super::module::Module;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub nlines: usize,
    pub modules: Vec<Module>,
    pub notes: Vec<Note>,
}

impl Track {
    pub fn new() -> Self {
        Self {
            name: "T01".to_string(),
            nlines: 16,
            modules: vec![],
            notes: vec![],
        }
    }

    pub fn compute_midi(&self, context: &mut ProcessTrackContext) {
        for note in self.notes.iter() {
            let time = note.line * 0x100 + note.delay as usize;
            if context.play_position.contains(&(time as i64)) {
                if let Some(key) = context.on_key {
                    context.event_list_input.push(Event::NoteOff(key));
                }
                // TODO time
                context
                    .event_list_input
                    .push(Event::NoteOn(note.key, note.velocity));
                context.on_key = Some(note.key);
            }
        }
    }

    pub fn process(&self, context: &mut ProcessTrackContext) -> Result<()> {
        self.compute_midi(context);
        let module_len = self.modules.len();
        for module_index in 0..module_len {
            self.process_module(context, module_index)?;
        }

        Ok(())
    }

    fn process_module(&self, context: &mut ProcessTrackContext, module_index: usize) -> Result<()> {
        let plugin = unsafe { &mut *(context.plugins[module_index].0 as *mut Plugin) };
        plugin.process(context)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn note(&self, line: usize) -> Option<&Note> {
        self.notes.iter().find(|note| note.line == line)
    }

    #[allow(dead_code)]
    pub fn note_mut(&mut self, line: usize) -> Option<&mut Note> {
        self.notes.iter_mut().find(|note| note.line == line)
    }
}
