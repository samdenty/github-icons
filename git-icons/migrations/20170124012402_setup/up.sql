CREATE TABLE repos (
  owner TEXT NOT NULL,
  repo TEXT NOT NULL,
  path TEXT NOT NULL,
  icon_path TEXT,
  UNIQUE(owner, repo, path),
  PRIMARY KEY(owner, repo, path)
);
CREATE TABLE icons (
  owner TEXT NOT NULL,
  repo TEXT NOT NULL,
  path TEXT NOT NULL,
  UNIQUE(owner, repo, path),
  PRIMARY KEY(owner, repo, path)
);
