//! Live announcements for assistive technology.
//!
//! egui hands a text field's value to AccessKit, but the AT-SPI text interface is built
//! from text runs and egui creates none for text edits, so a field's content never reaches
//! a Linux screen reader. A reader that also suppresses its own key echo -- Orca does for
//! password fields -- is then left with nothing to say at all.
//!
//! [`Announcer`] closes that gap: it keeps a `Status` node marked `Live::Polite` in the
//! accessibility tree, and AccessKit turns a name change on such a node into an AT-SPI
//! announcement that the reader speaks. The text is visible to assistive technology, so
//! keep secrets out of it: announce a count, never the content.
//!
//! ```no_run
//! # use egui::Ui;
//! # fn demo(ui: &mut Ui) {
//! let announcer = egui_elements::Announcer::new("credentials");
//! announcer.announce(ui, "Password, 3 characters");
//! # }
//! ```

use egui::{Id, Ui, accesskit};

/// A live status node whose text changes a screen reader speaks.
///
/// Call [`Self::announce`] every frame with the text that should currently be in the
/// tree, not only when it changes: the node has to stay in the tree for AccessKit to see
/// the change, and so the user can review it after the fact.
pub struct Announcer {
   id: Id,
}

impl Announcer {
   /// A node of its own, kept apart from every widget id.
   pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
      Self {
         id: Id::unique(("egui_elements_announcer", id_salt)),
      }
   }

   /// Put `text` in the live status node.
   ///
   /// A screen reader speaks `text` when it differs from the previous frame. The node is
   /// not drawn; it exists only in the accessibility tree.
   pub fn announce(&self, ui: &Ui, text: impl Into<String>) {
      let text = text.into();

      ui.ctx().accesskit_node_builder(self.id, |node| {
         node.set_role(accesskit::Role::Status);
         node.set_live(accesskit::Live::Polite);
         node.set_label(text);
      });
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use egui::accesskit::{Live, Node, Role};
   use egui::{Context, RawInput};

   fn frame(ctx: &Context, add: impl FnMut(&mut Ui)) -> Vec<Node> {
      ctx.enable_accesskit();

      let mut output = ctx.run_ui(RawInput::default(), add);
      let update = output
         .platform_output
         .accesskit_update
         .take()
         .expect("an AccessKit-enabled frame must produce a tree update");
      let nodes = update.nodes.into_iter().map(|(_, node)| node).collect();
      output.drop_without_applying_deltas();

      nodes
   }

   fn status_node(nodes: &[Node]) -> &Node {
      nodes
         .iter()
         .find(|node| node.role() == Role::Status)
         .expect("no live status node in the tree")
   }

   #[test]
   fn announce_keeps_a_polite_status_node() {
      let ctx = Context::default();
      let announcer = Announcer::new("status");
      let nodes = frame(&ctx, |ui| announcer.announce(ui, "3 characters"));
      let node = status_node(&nodes);

      assert_eq!(node.live(), Some(Live::Polite));
      assert_eq!(node.label(), Some("3 characters"));
   }

   #[test]
   fn announce_replaces_the_previous_text() {
      let ctx = Context::default();
      let announcer = Announcer::new("status");

      frame(&ctx, |ui| announcer.announce(ui, "3 characters"));
      let nodes = frame(&ctx, |ui| announcer.announce(ui, "4 characters"));

      assert_eq!(status_node(&nodes).label(), Some("4 characters"));
   }
}
