CREATE TABLE repos (
  owner TEXT NOT NULL,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  icon TEXT NOT NULL,
  PRIMARY KEY(owner, name, path)
);
CREATE TABLE icons (
  url TEXT NOT NULL,
  kind TEXT NOT NULL,
  sizes TEXT NOT NULL,
  type TEXT NOT NULL,
  UNIQUE(url),
  PRIMARY KEY(url)
);
