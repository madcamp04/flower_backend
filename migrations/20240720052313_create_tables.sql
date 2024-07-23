-- Add migration script here

CREATE TABLE Users_ (
  user_id INT AUTO_INCREMENT PRIMARY KEY,
  user_name VARCHAR(255) UNIQUE NOT NULL,
  user_email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE Sessions_ (
  session_id VARCHAR(255) PRIMARY KEY,
  user_id INT UNIQUE NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  is_persistent BOOLEAN DEFAULT false,
  FOREIGN KEY (user_id) REFERENCES Users_(user_id)
);

CREATE TABLE Groups_ (
  group_id INT AUTO_INCREMENT PRIMARY KEY,
  group_name VARCHAR(255) NOT NULL,
  owner_user_id INT NOT NULL,
  FOREIGN KEY (owner_user_id) REFERENCES Users_(user_id),
  UNIQUE (owner_user_id, group_name) -- no same group under same owner
);

CREATE TABLE GroupUserMapping_ (
  group_id INT,
  user_id INT,
  writeable BOOLEAN DEFAULT false,
  PRIMARY KEY (group_id, user_id),
  FOREIGN KEY (group_id) REFERENCES Groups_(group_id),
  FOREIGN KEY (user_id) REFERENCES Users_(user_id)
);

CREATE TABLE Tags_ (
  tag_id INT AUTO_INCREMENT PRIMARY KEY,
  group_id INT,
  tag_name VARCHAR(255) NOT NULL,
  tag_color VARCHAR(255) NOT NULL,
  FOREIGN KEY (group_id) REFERENCES Groups_(group_id),
  UNIQUE (group_id, tag_name) -- no same tags under same group
);

CREATE TABLE Projects_ (
  project_id INT AUTO_INCREMENT PRIMARY KEY,
  group_id INT,
  project_name VARCHAR(255) NOT NULL,
  project_description TEXT NOT NULL,
  FOREIGN KEY (group_id) REFERENCES Groups_(group_id),
  UNIQUE (group_id, project_name) -- no same project under same group
);

CREATE TABLE TagProjectMapping_ (
  tag_id INT,
  project_id INT,
  PRIMARY KEY (tag_id, project_id),
  FOREIGN KEY (tag_id) REFERENCES Tags_(tag_id),
  FOREIGN KEY (project_id) REFERENCES Projects_(project_id)
);

CREATE TABLE Tasks_ (
  task_id INT AUTO_INCREMENT PRIMARY KEY,
  project_id INT,
  worker_user_id INT,
  title VARCHAR(255) NOT NULL,
  description TEXT NOT NULL,
  start_time DATETIME NOT NULL,
  end_time DATETIME NOT NULL,
  FOREIGN KEY (project_id) REFERENCES Projects_(project_id),
  FOREIGN KEY (worker_user_id) REFERENCES Users_(user_id),
  UNIQUE (project_id, title) -- no same task under same project
);

CREATE TABLE Dependencies_ (
  prev_task_id INT,
  next_task_id INT,
  PRIMARY KEY (prev_task_id, next_task_id),
  FOREIGN KEY (prev_task_id) REFERENCES Tasks_(task_id),
  FOREIGN KEY (next_task_id) REFERENCES Tasks_(task_id)
);

-- Insert a new user into Users_ table
INSERT INTO Users_ (user_id, user_name, user_email, password_hash)
VALUES (1, 'admin', 'admin', 'admin');

INSERT INTO Sessions_ (session_id, user_id, expires_at, is_persistent)
VALUES ('admin123', 1, TIMESTAMP '2025-01-01 23:59:59', true);

INSERT INTO Users_ (user_id, user_name, user_email, password_hash)
VALUES (2, 'worker', 'worker', 'worker');

INSERT INTO Sessions_ (session_id, user_id, expires_at, is_persistent)
VALUES ('worker123', 2, TIMESTAMP '2025-01-01 23:59:59', true);
