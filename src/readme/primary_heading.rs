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
    // ugly hack to extract id https://github.com/causal-agent/ego-tree/pull/22
    let get_id = |node: &ElementRef| {
      let id_str = format!("{:?}", node.id());
      let res = regex!(r"\((\d+)\)").captures(&id_str).unwrap();
      res[1].parse::<usize>().unwrap()
    };

    let pos = self
      .primary_heading
      .as_ref()
      .map(|primary| {
        // if its before the primary heading
        if get_id(primary) > get_id(element) {
          Some(PrimaryHeadingPos::Preceding)
        } else {
          // or if its between the primary & next heading
          if self
            .next_heading
            .as_ref()
            .map(|next| get_id(next) > get_id(element))
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
