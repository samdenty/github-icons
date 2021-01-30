use crate::selector;
use scraper::{ElementRef, Html};

#[derive(Debug, Clone)]
enum PrimaryHeadingPos {
  Preceding,
  Trailing,
}

#[derive(Debug)]
pub struct PrimaryHeading<'a> {
  primary_heading: Option<ElementRef<'a>>,
  next_heading: Option<ElementRef<'a>>,
  first_img: Option<Option<PrimaryHeadingPos>>,
}

impl<'a> PrimaryHeading<'a> {
  pub fn new(document: &'a Html) -> Self {
    let mut headings = document.select(selector!("h1", "h2", "h3", "hr"));
    let first_heading = headings.next();
    let second_heading = headings.next();

    Self {
      primary_heading: first_heading,
      next_heading: second_heading,
      first_img: None,
    }
  }

  pub fn contains(&mut self, element: &ElementRef) -> bool {
    let pos = self
      .primary_heading
      .map(|primary| {
        // if its before the primary heading
        if primary.id() > element.id() {
          Some(PrimaryHeadingPos::Preceding)
        } else {
          // or if its between the primary & next heading
          if self
            .next_heading
            .map(|next| next.id() > element.id())
            .unwrap_or(true)
          {
            // but theres an image already before us...
            if let Some(Some(PrimaryHeadingPos::Preceding)) = &self.first_img {
              // then its out-of-bounds
              None
            } else {
              // else its trailing
              Some(PrimaryHeadingPos::Trailing)
            }
          } else {
            // if after both the first & second headings,
            // then its out-of-bounds
            None
          }
        }
      })
      .unwrap_or(Some(PrimaryHeadingPos::Preceding));

    if self.first_img.is_none() {
      self.first_img = Some(pos.clone());
    }

    pos.is_some()
  }
}
