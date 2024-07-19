CREATE TABLE Users (
    user_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(255) NOT NULL
);

CREATE TABLE ProjectManagers (
    pm_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    pm_name VARCHAR(255) NOT NULL,
    pm_user_id INTEGER,
    FOREIGN KEY (pm_user_id) REFERENCES Users(user_id)
);

CREATE TABLE Workers (
    worker_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    worker_name VARCHAR(255) NOT NULL,
    worker_user_id INTEGER,
    FOREIGN KEY (worker_user_id) REFERENCES Users(user_id)
);

CREATE TABLE PMWorkersMapping (
    pm_id INTEGER,
    worker_id INTEGER,
    PRIMARY KEY (pm_id, worker_id),
    FOREIGN KEY (pm_id) REFERENCES ProjectManagers(pm_id),
    FOREIGN KEY (worker_id) REFERENCES Workers(worker_id)
);

CREATE TABLE Tags (
    tag_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    pm_id INTEGER,
    tag_name VARCHAR(255) NOT NULL,
    FOREIGN KEY (pm_id) REFERENCES ProjectManagers(pm_id)
);

CREATE TABLE Projects (
    project_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    pm_id INTEGER,
    project_name VARCHAR(255) NOT NULL,
    FOREIGN KEY (pm_id) REFERENCES ProjectManagers(pm_id)
);

CREATE TABLE TagProjectMapping (
    tag_id INTEGER,
    project_id INTEGER,
    PRIMARY KEY (tag_id, project_id),
    FOREIGN KEY (tag_id) REFERENCES Tags(tag_id),
    FOREIGN KEY (project_id) REFERENCES Projects(project_id)
);

CREATE TABLE Tasks (
    task_id INTEGER PRIMARY KEY AUTO_INCREMENT,
    project_id INTEGER,
    worker_id INTEGER,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    FOREIGN KEY (project_id) REFERENCES Projects(project_id),
    FOREIGN KEY (worker_id) REFERENCES Workers(worker_id)
);

CREATE TABLE Dependencies (
    prev_task_id INTEGER,
    task_id INTEGER,
    PRIMARY KEY (prev_task_id, task_id),
    FOREIGN KEY (prev_task_id) REFERENCES Tasks(task_id),
    FOREIGN KEY (task_id) REFERENCES Tasks(task_id)
);
