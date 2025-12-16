# Komga API Documentation

This document describes the API endpoints and data transfer formats used by the Komga extension.

## Authentication

Supports API Key authentication via the `X-API-Key` request header.

Request header example:
```
User-Agent: TachiyomiKomga/{version}
X-API-Key: {api_key}
```

---

## API Endpoints

### 1. Series

#### Get Series List
```
GET /api/v1/series
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `search` | string | Search keyword |
| `page` | int | Page number (starts from 0) |
| `deleted` | boolean | Whether to include deleted content |
| `library_id` | string | Library ID (comma-separated for multiple) |
| `status` | string | Status filter: `ONGOING`, `ENDED`, `ABANDONED`, `HIATUS` |
| `read_status` | string | Read status: `UNREAD`, `IN_PROGRESS`, `READ` |
| `genre` | string | Genre filter (comma-separated for multiple) |
| `tag` | string | Tag filter (comma-separated for multiple) |
| `publisher` | string | Publisher filter (comma-separated for multiple) |
| `author` | string | Author filter (format: `name,role`) |
| `sort` | string | Sort order (e.g., `metadata.titleSort,asc`) |

**Sort Options:**
- `relevance` - Relevance
- `metadata.titleSort` - Sort by title
- `createdDate` - Creation date
- `lastModifiedDate` - Last modified date
- `random` - Random

**Response:** `PageWrapperDto<SeriesDto>`

#### Get Single Series Details
```
GET /api/v1/series/{seriesId}
```

**Response:** `SeriesDto`

#### Get Series Thumbnail
```
GET /api/v1/series/{seriesId}/thumbnail
```

**Response:** Image data

#### Get All Books in a Series
```
GET /api/v1/series/{seriesId}/books
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `unpaged` | boolean | Whether to disable pagination |
| `media_status` | string | Media status: `READY` |
| `deleted` | boolean | Whether to include deleted content |

**Response:** `PageWrapperDto<BookDto>`

---

### 2. Books

#### Search Books
```
GET /api/v1/books
```

**Query Parameters:** Similar to series search

**Response:** `PageWrapperDto<BookDto>`

#### Get Single Book Details
```
GET /api/v1/books/{bookId}
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `unpaged` | boolean | Whether to disable pagination |
| `media_status` | string | Media status |
| `deleted` | boolean | Whether to include deleted content |

**Response:** `BookDto`

#### Get Book Thumbnail
```
GET /api/v1/books/{bookId}/thumbnail
```

**Response:** Image data

#### Get Book Page List
```
GET /api/v1/books/{bookId}/pages
```

**Response:** `List<PageDto>`

#### Get Specific Page Image
```
GET /api/v1/books/{bookId}/pages/{pageNumber}
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `convert` | string | Convert format (e.g., `png`) |

**Supported Image Formats:**
- `image/jpeg`
- `image/png`
- `image/gif`
- `image/webp`
- `image/jxl`
- `image/heif`
- `image/avif`

**Response:** Image data

---

---

### Libraries

#### Get All Libraries
```
GET /api/v1/libraries
```

**Response:** `List<LibraryDto>`

---

### 6. Metadata Filter Options

#### Get All Genres
```
GET /api/v1/genres
```

**Response:** `Set<String>`

#### Get All Tags
```
GET /api/v1/tags
```

**Response:** `Set<String>`

#### Get All Publishers
```
GET /api/v1/publishers
```

**Response:** `Set<String>`

#### Get All Authors
```
GET /api/v1/authors
```

**Response:** `List<AuthorDto>`

---

## Data Transfer Objects (DTO)

### PageWrapperDto<T>
Pagination wrapper

```json
{
  "content": [],         // T[] - Content array
  "empty": false,        // boolean - Whether empty
  "first": true,         // boolean - Whether first page
  "last": false,         // boolean - Whether last page
  "number": 0,           // long - Current page number
  "numberOfElements": 20,// long - Number of elements on current page
  "size": 20,            // long - Page size
  "totalElements": 100,  // long - Total elements
  "totalPages": 5        // long - Total pages
}
```

---

### SeriesDto
Series information

```json
{
  "id": "string",
  "libraryId": "string",
  "name": "string",
  "created": "2024-01-01T00:00:00Z",      // nullable
  "lastModified": "2024-01-01T00:00:00Z", // nullable
  "fileLastModified": "2024-01-01T00:00:00Z",
  "booksCount": 10,
  "metadata": {
    "status": "ONGOING",          // ONGOING | ENDED | ABANDONED | HIATUS
    "created": "2024-01-01T00:00:00Z",
    "lastModified": "2024-01-01T00:00:00Z",
    "title": "string",
    "titleSort": "string",
    "summary": "string",
    "summaryLock": false,
    "readingDirection": "LEFT_TO_RIGHT",
    "readingDirectionLock": false,
    "publisher": "string",
    "publisherLock": false,
    "ageRating": 0,               // nullable
    "ageRatingLock": false,
    "language": "en",
    "languageLock": false,
    "genres": ["Action", "Adventure"],
    "genresLock": false,
    "tags": ["tag1", "tag2"],
    "tagsLock": false,
    "totalBookCount": 10          // nullable
  },
  "booksMetadata": {
    "authors": [
      {
        "name": "Author Name",
        "role": "writer"
      }
    ],
    "tags": ["tag1"],
    "releaseDate": "2024-01-01",  // nullable
    "summary": "string",
    "summaryNumber": "string",
    "created": "2024-01-01T00:00:00Z",
    "lastModified": "2024-01-01T00:00:00Z"
  }
}
```

---

### BookDto
Book information

```json
{
  "id": "string",
  "seriesId": "string",
  "seriesTitle": "string",
  "name": "string",
  "number": 1.0,
  "created": "2024-01-01T00:00:00Z",      // nullable
  "lastModified": "2024-01-01T00:00:00Z", // nullable
  "fileLastModified": "2024-01-01T00:00:00Z",
  "sizeBytes": 1024000,
  "size": "1 MB",
  "media": {
    "status": "READY",
    "mediaType": "application/zip",
    "pagesCount": 100,
    "mediaProfile": "DIVINA",      // DIVINA | EPUB
    "epubDivinaCompatible": false
  },
  "metadata": {
    "title": "Chapter 1",
    "titleLock": false,
    "summary": "string",
    "summaryLock": false,
    "number": "1",
    "numberLock": false,
    "numberSort": 1.0,
    "numberSortLock": false,
    "releaseDate": "2024-01-01",   // nullable
    "releaseDateLock": false,
    "authors": [
      {
        "name": "Author Name",
        "role": "writer"
      }
    ],
    "authorsLock": false,
    "tags": ["tag1"],
    "tagsLock": false
  }
}
```

---

### PageDto
Page information

```json
{
  "number": 1,
  "fileName": "page001.jpg",
  "mediaType": "image/jpeg"
}
```

---

### LibraryDto
Library information

```json
{
  "id": "string",
  "name": "My Library"
}
```

---

---

### AuthorDto
Author information

```json
{
  "name": "Author Name",
  "role": "writer"  // writer | penciller | translator | etc.
}
```

---

## Author Role Types

| Role | Description |
|------|-------------|
| `writer` | Writer/Author |
| `penciller` | Penciller/Artist |
| `translator` | Translator |

---

## Status Enumerations

### Series Status
| Value | Description |
|-------|-------------|
| `ONGOING` | Ongoing |
| `ENDED` | Ended |
| `ABANDONED` | Abandoned |
| `HIATUS` | On Hiatus |

### Read Status
| Value | Description |
|-------|-------------|
| `UNREAD` | Unread |
| `IN_PROGRESS` | In Progress |
| `READ` | Read |

### Media Status
| Value | Description |
|-------|-------------|
| `READY` | Ready |

### Media Profile
| Value | Description |
|-------|-------------|
| `DIVINA` | Standard comic format |
| `EPUB` | EPUB e-book format |

---

## Usage Examples

### Search Series
```http
GET /api/v1/series?search=manga&page=0&library_id=lib1,lib2&status=ONGOING&sort=metadata.titleSort,asc&deleted=false
```

### Get Popular Series (sorted by title)
```http
GET /api/v1/series?page=0&sort=metadata.titleSort,asc&deleted=false
```

### Get Latest Updates (sorted by modified date descending)
```http
GET /api/v1/series?page=0&sort=lastModifiedDate,desc&deleted=false
```

### Get Chapter List
```http
GET /api/v1/series/{seriesId}/books?unpaged=true&media_status=READY&deleted=false
```

### Get Page Image
```http
GET /api/v1/books/{bookId}/pages/{pageNumber}
Accept: image/*,*/*;q=0.8
```

If the image format is not supported, add the convert parameter:
```http
GET /api/v1/books/{bookId}/pages/{pageNumber}?convert=png
```
