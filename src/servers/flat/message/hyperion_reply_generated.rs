// automatically generated by the FlatBuffers compiler, do not modify



use std::mem;
use std::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::EndianScalar;

#[allow(unused_imports, dead_code)]
pub mod hyperionnet {

  use std::mem;
  use std::cmp::Ordering;

  extern crate flatbuffers;
  use self::flatbuffers::EndianScalar;

pub enum ReplyOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct Reply<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Reply<'a> {
    type Inner = Reply<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self { _tab: flatbuffers::Table { buf, loc } }
    }
}

impl<'a> Reply<'a> {
    #[inline]
    pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Reply { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args ReplyArgs<'args>) -> flatbuffers::WIPOffset<Reply<'bldr>> {
      let mut builder = ReplyBuilder::new(_fbb);
      builder.add_registered(args.registered);
      builder.add_video(args.video);
      if let Some(x) = args.error { builder.add_error(x); }
      builder.finish()
    }

    pub const VT_ERROR: flatbuffers::VOffsetT = 4;
    pub const VT_VIDEO: flatbuffers::VOffsetT = 6;
    pub const VT_REGISTERED: flatbuffers::VOffsetT = 8;

  #[inline]
  pub fn error(&self) -> Option<&'a str> {
    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(Reply::VT_ERROR, None)
  }
  #[inline]
  pub fn video(&self) -> i32 {
    self._tab.get::<i32>(Reply::VT_VIDEO, Some(-1)).unwrap()
  }
  #[inline]
  pub fn registered(&self) -> i32 {
    self._tab.get::<i32>(Reply::VT_REGISTERED, Some(-1)).unwrap()
  }
}

impl flatbuffers::Verifiable for Reply<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<flatbuffers::ForwardsUOffset<&str>>(&"error", Self::VT_ERROR, false)?
     .visit_field::<i32>(&"video", Self::VT_VIDEO, false)?
     .visit_field::<i32>(&"registered", Self::VT_REGISTERED, false)?
     .finish();
    Ok(())
  }
}
pub struct ReplyArgs<'a> {
    pub error: Option<flatbuffers::WIPOffset<&'a str>>,
    pub video: i32,
    pub registered: i32,
}
impl<'a> Default for ReplyArgs<'a> {
    #[inline]
    fn default() -> Self {
        ReplyArgs {
            error: None,
            video: -1,
            registered: -1,
        }
    }
}
pub struct ReplyBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ReplyBuilder<'a, 'b> {
  #[inline]
  pub fn add_error(&mut self, error: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Reply::VT_ERROR, error);
  }
  #[inline]
  pub fn add_video(&mut self, video: i32) {
    self.fbb_.push_slot::<i32>(Reply::VT_VIDEO, video, -1);
  }
  #[inline]
  pub fn add_registered(&mut self, registered: i32) {
    self.fbb_.push_slot::<i32>(Reply::VT_REGISTERED, registered, -1);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ReplyBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ReplyBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Reply<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl std::fmt::Debug for Reply<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut ds = f.debug_struct("Reply");
      ds.field("error", &self.error());
      ds.field("video", &self.video());
      ds.field("registered", &self.registered());
      ds.finish()
  }
}
#[inline]
#[deprecated(since="2.0.0", note="Deprecated in favor of `root_as...` methods.")]
pub fn get_root_as_reply<'a>(buf: &'a [u8]) -> Reply<'a> {
  unsafe { flatbuffers::root_unchecked::<Reply<'a>>(buf) }
}

#[inline]
#[deprecated(since="2.0.0", note="Deprecated in favor of `root_as...` methods.")]
pub fn get_size_prefixed_root_as_reply<'a>(buf: &'a [u8]) -> Reply<'a> {
  unsafe { flatbuffers::size_prefixed_root_unchecked::<Reply<'a>>(buf) }
}

#[inline]
/// Verifies that a buffer of bytes contains a `Reply`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_reply_unchecked`.
pub fn root_as_reply(buf: &[u8]) -> Result<Reply, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root::<Reply>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `Reply` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_reply_unchecked`.
pub fn size_prefixed_root_as_reply(buf: &[u8]) -> Result<Reply, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root::<Reply>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `Reply` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_reply_unchecked`.
pub fn root_as_reply_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<Reply<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root_with_opts::<Reply<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `Reply` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_reply_unchecked`.
pub fn size_prefixed_root_as_reply_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<Reply<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root_with_opts::<Reply<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a Reply and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `Reply`.
pub unsafe fn root_as_reply_unchecked(buf: &[u8]) -> Reply {
  flatbuffers::root_unchecked::<Reply>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed Reply and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `Reply`.
pub unsafe fn size_prefixed_root_as_reply_unchecked(buf: &[u8]) -> Reply {
  flatbuffers::size_prefixed_root_unchecked::<Reply>(buf)
}
#[inline]
pub fn finish_reply_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<Reply<'a>>) {
  fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_reply_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<Reply<'a>>) {
  fbb.finish_size_prefixed(root, None);
}
}  // pub mod hyperionnet
