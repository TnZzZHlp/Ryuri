# Requirements Document

## Introduction

本文档定义了一个 Web 端阅读软件的需求规格。该系统允许用户导入、管理和阅读本地漫画和小说文件，提供流畅的阅读体验。系统采用 Vue.js 前端、Rust 后端和 SQLite 数据库的技术栈。

## Glossary

- **Reader_System**: 阅读软件系统，负责漫画和小说的导入、管理和阅读功能
- **Content**: 内容，漫画或小说的统称
- **Content_Type**: 内容类型，分为 Comic（漫画）和 Novel（小说）两种
- **Comic**: 漫画，对应 Scan_Path 下的一个以漫画名称命名的文件夹，包含多个 Chapter
- **Novel**: 小说，对应 Scan_Path 下的一个以小说名称命名的文件夹，包含多个 Chapter
- **Chapter**: 章节，内容文件夹内的一个压缩包文件，代表一话/一卷（漫画）或一章/一卷（小说）
- **Library**: 内容库，一个独立的内容集合，可关联多个文件系统目录，需指定 Content_Type，可配置自动扫描策略
- **Scan_Path**: 扫描路径，Library 关联的文件系统目录路径，一个 Library 可以有多个 Scan_Path
- **Scan_Interval**: 扫描间隔，Library 的定时扫描时间间隔（分钟），设为 0 表示禁用定时扫描
- **Watch_Mode**: 监听模式，是否启用文件系统变化监听，实时检测新增或删除的内容
- **Reading_Progress**: 阅读进度，记录用户在某个内容中的阅读位置
- **Comic_Archive**: 漫画压缩包，支持的格式包括 ZIP、CBZ、CBR、RAR，内含图片文件
- **Novel_Archive**: 小说压缩包，支持的格式包括 ZIP、EPUB、TXT，内含文本文件
- **Thumbnail**: 缩略图，漫画封面或小说封面的预览图片
- **Metadata**: 元数据，包括作品标题、作者、简介、评分、标签等信息
- **Bangumi_API**: Bangumi.tv 提供的 API 服务，用于获取作品元数据
- **User**: 用户，系统的使用者，拥有独立的阅读进度和设置
- **JWT**: JSON Web Token，用于用户认证的无状态令牌

## Requirements

### Requirement 1: 多库管理

**User Story:** As a user, I want to create and manage multiple content libraries with multiple scan paths and automatic scanning options, so that I can organize my comics and novels into different collections and keep them automatically updated.

#### Acceptance Criteria

1. WHEN a user creates a new Library THEN the Reader_System SHALL store the Library name, Content_Type, Scan_Interval, and Watch_Mode in the database
2. WHEN a user adds a Scan_Path to a Library THEN the Reader_System SHALL associate the path with that Library and store it in the database
3. WHEN a user removes a Scan_Path from a Library THEN the Reader_System SHALL remove the path association and delete Content imported from that path
4. WHEN a user views the library list THEN the Reader_System SHALL display all Libraries with their names, Content_Type, Scan_Path count, content counts, and scan settings
5. WHEN a user selects a Library THEN the Reader_System SHALL display all Content belonging to that Library from all associated Scan_Paths
6. WHEN a user deletes a Library THEN the Reader_System SHALL remove the Library record, all Scan_Path associations, and all associated Content records from the database
7. WHEN a user renames a Library THEN the Reader_System SHALL update the Library name in the database
8. WHEN a user sets a Scan_Interval for a Library THEN the Reader_System SHALL automatically scan the Library at the specified interval
9. WHEN a user enables Watch_Mode for a Library THEN the Reader_System SHALL monitor the Scan_Paths for file system changes and update Content accordingly
10. WHILE Watch_Mode is enabled THEN the Reader_System SHALL detect new Content folders and import them automatically
11. WHILE Watch_Mode is enabled THEN the Reader_System SHALL detect deleted Content folders and remove the corresponding records

### Requirement 2: 内容导入与管理

**User Story:** As a user, I want to import and manage content within a library, so that I can organize and access my comics and novels easily.

#### Acceptance Criteria

1. WHEN a user triggers a scan on a Library THEN the Reader_System SHALL scan all associated Scan_Paths, identify Content folders by their directory names, and import each folder as Content based on the Library Content_Type
2. WHEN a Comic folder is scanned THEN the Reader_System SHALL identify all Comic_Archive files within the folder as Chapters and sort them by filename
3. WHEN a Novel folder is scanned THEN the Reader_System SHALL identify all Novel_Archive files within the folder as Chapters and sort them by filename
4. WHEN Content is imported THEN the Reader_System SHALL use the folder name as the Content title and count the number of Chapters
5. WHEN a Comic is imported THEN the Reader_System SHALL generate a Thumbnail from the first page of the first Chapter
6. WHEN a Novel is imported THEN the Reader_System SHALL generate a default Thumbnail or extract cover image if available
7. WHEN Content is imported THEN the Reader_System SHALL record which Scan_Path the Content was imported from
8. WHEN a user views a Library THEN the Reader_System SHALL display all imported Content with their Thumbnails, titles, and Chapter counts in a grid layout
9. WHEN a user deletes Content from a Library THEN the Reader_System SHALL remove the Content record from the database and delete associated Thumbnails
10. WHEN a user searches for Content by title within a Library THEN the Reader_System SHALL return all Content whose titles contain the search keyword

### Requirement 3: 漫画阅读

**User Story:** As a user, I want to read comics with a smooth experience, so that I can enjoy my comics comfortably.

#### Acceptance Criteria

1. WHEN a user opens a Comic THEN the Reader_System SHALL display the Chapter list and allow the user to select a Chapter to read
2. WHEN a user opens a Comic Chapter THEN the Reader_System SHALL load and display all images in the Chapter in sequential order
3. WHEN a user navigates to the next or previous page THEN the Reader_System SHALL display the corresponding image within 200 milliseconds
4. WHEN a user reaches the end of a Chapter THEN the Reader_System SHALL prompt the user to continue to the next Chapter
5. WHEN a user closes the comic reader THEN the Reader_System SHALL save the current Reading_Progress to the database

### Requirement 4: 小说阅读

**User Story:** As a user, I want to read novels with a comfortable experience, so that I can enjoy my novels easily.

#### Acceptance Criteria

1. WHEN a user opens a Novel THEN the Reader_System SHALL display the Chapter list and allow the user to select a Chapter to read
2. WHEN a user opens a Novel Chapter THEN the Reader_System SHALL load and display the text content with proper formatting
3. WHEN a user scrolls through the text THEN the Reader_System SHALL smoothly render the content without lag
4. WHEN a user reaches the end of a Chapter THEN the Reader_System SHALL prompt the user to continue to the next Chapter
5. WHEN a user closes the novel reader THEN the Reader_System SHALL save the current Reading_Progress to the database

### Requirement 5: 阅读进度管理

**User Story:** As a user, I want my reading progress to be saved automatically, so that I can continue reading from where I left off.

#### Acceptance Criteria

1. WHEN a user opens previously read Content THEN the Reader_System SHALL offer to resume from the last Reading_Progress
2. WHEN a user reads a page or scrolls through text THEN the Reader_System SHALL update the Reading_Progress in memory
3. WHEN the Reading_Progress changes THEN the Reader_System SHALL persist the progress to the database within 5 seconds
4. WHEN a user views the Library THEN the Reader_System SHALL display the reading percentage for each Content that has Reading_Progress

### Requirement 6: 阅读设置

**User Story:** As a user, I want to customize my reading experience, so that I can read content in my preferred way.

#### Acceptance Criteria

1. WHEN a user selects a comic reading mode THEN the Reader_System SHALL switch between single-page mode and continuous scroll mode
2. WHEN a user adjusts the zoom level for comics THEN the Reader_System SHALL scale the current image to the specified percentage
3. WHEN a user enables fit-to-width mode for comics THEN the Reader_System SHALL automatically scale images to fit the viewport width
4. WHEN a user changes reading direction for comics THEN the Reader_System SHALL support both left-to-right and right-to-left page navigation
5. WHEN a user adjusts font size for novels THEN the Reader_System SHALL scale the text to the specified size
6. WHEN a user changes the theme for novels THEN the Reader_System SHALL apply the selected color scheme to the reading interface

### Requirement 7: 后端 API

**User Story:** As a frontend application, I want to communicate with the backend through a RESTful API, so that I can perform all content management and reading operations.

#### Acceptance Criteria

1. WHEN the frontend requests the content list THEN the Reader_System SHALL return a JSON array containing all Content with their metadata
2. WHEN the frontend requests comic images THEN the Reader_System SHALL extract and serve images from the Comic_Archive with appropriate caching headers
3. WHEN the frontend requests novel text THEN the Reader_System SHALL extract and serve text content from the Novel_Archive
4. WHEN the frontend sends a progress update THEN the Reader_System SHALL validate the data and persist it to the SQLite database
5. IF the backend receives an invalid request THEN the Reader_System SHALL return an appropriate HTTP error code with a descriptive error message

### Requirement 8: 元数据刮削

**User Story:** As a user, I want to automatically fetch metadata from Bangumi.tv when new content is imported, so that I can have rich information like author, description, ratings, and tags without manual intervention.

#### Acceptance Criteria

1. WHEN new Content is imported during a scan THEN the Reader_System SHALL automatically search Bangumi_API using the Content title and fetch Metadata from the first matching result
2. WHEN automatic scraping succeeds THEN the Reader_System SHALL store the Metadata JSON blob directly in the database and associate it with the Content
3. WHEN automatic scraping fails or returns no results THEN the Reader_System SHALL continue importing the Content without Metadata and log the failure
4. WHEN a user manually triggers metadata scraping for Content THEN the Reader_System SHALL search Bangumi_API and display matching entries for the user to select
5. WHEN a user selects a Bangumi entry THEN the Reader_System SHALL fetch detailed Metadata and update the Content record
6. WHEN a user views Content with Metadata THEN the Reader_System SHALL display the Metadata alongside the Content information
7. WHEN a user manually edits Metadata THEN the Reader_System SHALL update the stored Metadata in the database
8. IF Bangumi_API is unavailable THEN the Reader_System SHALL continue the import process without Metadata and allow the user to retry scraping later

### Requirement 9: 用户管理

**User Story:** As a user, I want to have my own account, so that my reading progress and settings are personal and separate from other users.

#### Acceptance Criteria

1. WHEN a user registers THEN the Reader_System SHALL create a new User record with username and hashed password
2. WHEN a user logs in with valid credentials THEN the Reader_System SHALL create a Session and return an authentication token
3. WHEN a user logs in with invalid credentials THEN the Reader_System SHALL reject the login and return an error message
4. WHEN a user accesses protected resources THEN the Reader_System SHALL verify the JWT token before allowing access
5. WHEN a user updates their password THEN the Reader_System SHALL hash and store the new password
6. WHEN Reading_Progress is saved THEN the Reader_System SHALL associate the progress with the current User

### Requirement 10: 数据持久化

**User Story:** As a system administrator, I want all data to be stored reliably, so that user data is preserved across application restarts.

#### Acceptance Criteria

1. WHEN the Reader_System starts THEN the system SHALL initialize the SQLite database and create required tables if they do not exist
2. WHEN Content data is stored THEN the Reader_System SHALL serialize the data to JSON format for storage in SQLite
3. WHEN Content data is retrieved THEN the Reader_System SHALL deserialize the JSON data back to structured objects
4. WHEN the Reader_System stores or retrieves data THEN the system SHALL use a pretty printer for JSON serialization to enable human-readable storage and round-trip consistency
