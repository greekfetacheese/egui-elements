//! Static model and sample data for the Telegram demo.
//!
//! Nine chats with short conversations, so switching rows looks alive. No
//! chat is created at runtime and no message is generated: everything is
//! `&'static` so the UI can borrow it without cloning.

/// `Chat::id` of Eva Summer, the chat selected at start-up.
pub const EVA_ID: usize = 3;

/// One row in the chat list.
pub struct Chat {
   /// Stable id, also the `messages` key.
   pub id: usize,
   /// Display name.
   pub title: &'static str,
   /// Two letters for the fallback avatar plate.
   pub initials: &'static str,
   /// The contact is online (paints a presence dot).
   pub online: bool,
   /// A group chat (previews carry an author).
   pub group: bool,
   /// Last-message preview.
   pub preview: Preview,
   /// Last-message time label.
   pub time: &'static str,
   /// Unread count; `0` hides the badge.
   pub unread: u32,
   /// Notifications muted (badge and preview read as muted).
   pub muted: bool,
   /// Pinned chat (pin glyph in the title line).
   pub pinned: bool,
   /// The last outgoing message was read (double tick in the meta column).
   pub preview_read: bool,
}

/// Last-message preview shapes, each rendering differently in the chat row.
#[derive(Clone, Copy)]
pub enum Preview {
   /// A plain message body.
   Text(&'static str),
   /// A group message: author and body.
   Author(&'static str, &'static str),
   /// A sticker, named.
   Sticker(&'static str),
   /// An unsent draft.
   Draft(&'static str),
}

impl Preview {
   /// One-line rendering for the chat row and the live filter.
   pub fn line(&self) -> String {
      match self {
         Preview::Text(text) => (*text).to_owned(),
         Preview::Author(author, text) => format!("{author}: {text}"),
         Preview::Sticker(name) => format!("{name} sticker"),
         Preview::Draft(text) => format!("Draft: {text}"),
      }
   }

   /// Whether the preview is an unsent draft.
   pub fn is_draft(&self) -> bool {
      matches!(self, Preview::Draft(_))
   }
}

/// What a message carries.
#[derive(Clone, Copy)]
pub enum MsgBody {
   /// Day separator pill.
   Day(&'static str),
   /// Plain text.
   Text(&'static str),
   /// Procedural photo with a caption.
   Photo {
      /// Caption under the image.
      caption: &'static str,
      /// Deterministic geometry seed for `paint::photo`.
      seed: u32,
   },
   /// Voice message.
   Voice {
      /// Duration label, e.g. `"0:07"`.
      duration: &'static str,
      /// Deterministic seed for `paint::waveform`.
      seed: u32,
   },
   /// Attached document.
   File {
      /// File name.
      name: &'static str,
      /// Human-readable size.
      size: &'static str,
   },
   /// Sticker, named.
   Sticker(&'static str),
}

/// Delivery state of an outgoing message.
#[derive(Clone, Copy)]
pub enum MsgStatus {
   /// Still sending.
   Sending,
   /// Delivered, not read.
   Sent,
   /// Read by the peer (double tick).
   Read,
}

/// Quoted message shown above a reply.
#[derive(Clone, Copy)]
pub struct Quote {
   /// Author of the quoted message.
   pub author: &'static str,
   /// Quoted body, shortened.
   pub text: &'static str,
}

/// One message in a conversation.
#[derive(Clone, Copy)]
pub struct Msg {
   /// Sent by the demo user.
   pub from_me: bool,
   /// Group author, `None` for a direct chat or an own message.
   pub author: Option<&'static str>,
   /// Payload.
   pub body: MsgBody,
   /// Time label; empty for a day pill.
   pub time: &'static str,
   /// Delivery state.
   pub status: MsgStatus,
   /// Reply target, if any.
   pub quote: Option<Quote>,
   /// Painted in `colors.error` by the conversation pane.
   pub failed: bool,
}

/// Every chat in list order.
pub fn chats() -> &'static [Chat] {
   CHATS
}

/// Conversation for `chat_id`; empty for an unknown id.
pub fn messages(chat_id: usize) -> &'static [Msg] {
   match chat_id {
      1 => SAVED_MSGS,
      2 => DEATH_STAR_MSGS,
      EVA_ID => EVA_MSGS,
      4 => LENA_MSGS,
      5 => MOM_MSGS,
      6 => PANDAS_MSGS,
      7 => PIRATES_MSGS,
      8 => MAX_MSGS,
      9 => LEE_MSGS,
      _ => &[],
   }
}

const CHATS: &[Chat] = &[
   Chat {
      id: 1,
      title: "Saved Messages",
      initials: "SM",
      online: false,
      group: false,
      preview: Preview::Draft("note for the demo"),
      time: "9:41 PM",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: false,
   },
   Chat {
      id: 2,
      title: "Death Star 3.0",
      initials: "DS",
      online: false,
      group: false,
      preview: Preview::Author("Darth", "I'll trust you. For now."),
      time: "7:15 PM",
      unread: 0,
      muted: false,
      pinned: true,
      preview_read: false,
   },
   Chat {
      id: EVA_ID,
      title: "Eva Summer",
      initials: "ES",
      online: true,
      group: false,
      preview: Preview::Text("Reminds me of a Chinese prove…"),
      time: "11:28 PM",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: false,
   },
   Chat {
      id: 4,
      title: "Lena Oxton",
      initials: "LO",
      online: true,
      group: false,
      preview: Preview::Sticker("Cheering fox"),
      time: "9:17 PM",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: true,
   },
   Chat {
      id: 5,
      title: "Mom",
      initials: "M",
      online: false,
      group: false,
      preview: Preview::Text("Don't forget your blaster and helmet"),
      time: "8:02 PM",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: false,
   },
   Chat {
      id: 6,
      title: "Pandas HQ",
      initials: "PH",
      online: false,
      group: true,
      preview: Preview::Author("Eva", "Photo"),
      time: "1:14 AM",
      unread: 3,
      muted: true,
      pinned: false,
      preview_read: false,
   },
   Chat {
      id: 7,
      title: "Old Pirates",
      initials: "OP",
      online: false,
      group: true,
      preview: Preview::Author("Jack", "Yo-ho-ho, all aboard!"),
      time: "0:02 AM",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: false,
   },
   Chat {
      id: 8,
      title: "Max Bright",
      initials: "MB",
      online: false,
      group: false,
      preview: Preview::Text("Coffee time? ☕"),
      time: "Mon",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: true,
   },
   Chat {
      id: 9,
      title: "Lee",
      initials: "L",
      online: false,
      group: false,
      preview: Preview::Text("We can call it Galaxy Star 7 ;)"),
      time: "Mon",
      unread: 0,
      muted: false,
      pinned: false,
      preview_read: false,
   },
];

const SAVED_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 2"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text("Demo notes: wallpaper from paint::wallpaper, 72 px rows, 9 px pill."),
      time: "9:38 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Outgoing bubbles are mix(widget_bg, accent, 0.32) with a hairline."),
      time: "9:41 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
];

const DEATH_STAR_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 3"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: Some("Darth"),
      body: MsgBody::Text("The new targeting computer is ready."),
      time: "10:58 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: Some("Darth"),
      body: MsgBody::Text("I'll trust you. For now."),
      time: "11:02 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
];

const EVA_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 4"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Photo {
         caption: "I finally visited Earth.. The nature here is fantastic!",
         seed: 7,
      },
      time: "11:23 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Voice {
         duration: "0:07",
         seed: 3,
      },
      time: "11:24 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::File {
         name: "First Impression.pdf",
         size: "11.2 KB",
      },
      time: "11:27 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text(
         "Twenty years from now you will be more disappointed by the things that you didn't do than by the ones you did do.",
      ),
      time: "11:28 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Mark Twain said that"),
      time: "11:28 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text(
         "Reminds me of a Chinese proverb: the best time to plant a tree was twenty years ago. The second best time is now.",
      ),
      time: "11:28 PM",
      status: MsgStatus::Sent,
      quote: Some(Quote {
         author: "Mary Sue",
         text: "Twenty years from now you will be more disappointed by the things that you didn't do…",
      }),
      failed: false,
   },
];

const LENA_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 3"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Sticker("Cheering fox"),
      time: "10:51 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Perfect timing — the demo ships today."),
      time: "10:54 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Cheers!"),
      time: "10:57 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
];

const MOM_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 3"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text("Don't forget your blaster and helmet"),
      time: "9:04 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("They are already in the ship, Mom."),
      time: "9:09 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text("And eat something."),
      time: "9:12 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
];

const PANDAS_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("November 2"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: Some("Eva"),
      body: MsgBody::Photo {
         caption: "Look at this panda.",
         seed: 4,
      },
      time: "8:31 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: Some("Kai"),
      body: MsgBody::Text("That is a very round panda."),
      time: "8:36 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Photo of the year."),
      time: "8:40 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
];

const PIRATES_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("Yesterday"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: Some("Jack"),
      body: MsgBody::Text("Yo-ho-ho, all aboard!"),
      time: "7:12 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Setting sail at dawn."),
      time: "7:15 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: true,
   },
   Msg {
      from_me: false,
      author: Some("Anne"),
      body: MsgBody::Text("The map is upside down again."),
      time: "7:20 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
];

const MAX_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("Yesterday"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Coffee time? ☕"),
      time: "4:02 PM",
      status: MsgStatus::Read,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text("Always. Same place?"),
      time: "4:05 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
];

const LEE_MSGS: &[Msg] = &[
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Day("Monday"),
      time: "",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: false,
      author: None,
      body: MsgBody::Text("We can call it Galaxy Star 7 ;)"),
      time: "6:20 PM",
      status: MsgStatus::Sent,
      quote: None,
      failed: false,
   },
   Msg {
      from_me: true,
      author: None,
      body: MsgBody::Text("Naming rights are yours."),
      time: "6:22 PM",
      status: MsgStatus::Sending,
      quote: None,
      failed: false,
   },
];
