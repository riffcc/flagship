# Structures Documentation

This document details the structure system in Riff.CC - a generic hierarchical organization system for ANY type of content.

## Overview

Structures are completely generic containers that can represent ANY organizational entity and form arbitrary hierarchies. They are designed to enable efficient PeerBit queries across complex relationships.

A structure can be:
- **Artist** (containing albums, songs, collaborations)
- **Author** (containing book series, individual books)
- **Director** (containing filmographies, TV shows)
- **Actor** (containing performances across different media)
- **TV Show** (containing seasons)
- **Season** (containing episodes)  
- **Album** (containing tracks)
- **Book Series** (containing volumes)
- **Course** (containing lessons)
- **Any other organizational concept**

## Data Model

### Structure Schema

```typescript
interface Structure {
  id: string;              // Unique identifier
  type: string;            // Completely arbitrary - 'artist', 'album', 'tv-show', 'season', 'course', etc.
  title: string;           // Display name
  description?: string;    // Optional description
  thumbnail?: string;      // CID for thumbnail image
  parentId?: string;       // References parent structure (completely optional)
  categoryId?: string;     // Optional reference to content category
  categorySlug?: string;   // Optional category slug
  createdAt: Date;
  updatedAt: Date;
}
```

### Content Relationship

Content (releases) can reference structures through metadata:

```typescript
interface Release {
  // ... standard release fields
  metadata: {
    structureId?: string;     // References ANY structure
    parentStructureId?: string; // References parent structure
    // ... other metadata
  };
}
```

## Hierarchy Examples

### Music Organization
```
Artist: "Radiohead" (type: 'artist')
├── Album: "OK Computer" (type: 'album', parentId: radiohead-id)
│   ├── Track: "Paranoid Android" (metadata.structureId: ok-computer-id)
│   └── Track: "Karma Police" (metadata.structureId: ok-computer-id)
└── Album: "In Rainbows" (type: 'album', parentId: radiohead-id)
    ├── Track: "15 Step" (metadata.structureId: in-rainbows-id)
    └── Track: "Bodysnatchers" (metadata.structureId: in-rainbows-id)
```

### TV Organization
```
TV Show: "Breaking Bad" (type: 'tv-show')
├── Season: "Season 1" (type: 'season', parentId: breaking-bad-id)
│   ├── Episode: "Pilot" (metadata.structureId: season-1-id)
│   └── Episode: "Cat's in the Bag..." (metadata.structureId: season-1-id)
└── Season: "Season 2" (type: 'season', parentId: breaking-bad-id)
```

### Book Series Organization
```
Author: "J.K. Rowling" (type: 'author')
└── Series: "Harry Potter" (type: 'book-series', parentId: jk-rowling-id)
    ├── Book: "Philosopher's Stone" (metadata.structureId: harry-potter-id)
    └── Book: "Chamber of Secrets" (metadata.structureId: harry-potter-id)
```

## Query Patterns

The power of structures lies in efficient PeerBit queries across hierarchical relationships:

### Find all content under a structure
```typescript
// Find all tracks in an album
const tracks = await site.releases.query({
  'metadata.structureId': albumId
});

// Find all episodes in a season
const episodes = await site.releases.query({
  'metadata.structureId': seasonId
});
```

### Find all child structures
```typescript
// Find all albums by an artist
const albums = await site.structures.query({
  parentId: artistId
});

// Find all seasons of a TV show
const seasons = await site.structures.query({
  parentId: tvShowId
});
```

### Multi-level queries
```typescript
// Find everything by an artist (albums + standalone tracks)
const artistAlbums = await site.structures.query({ parentId: artistId });
const directTracks = await site.releases.query({ 'metadata.structureId': artistId });

// Get all tracks from all albums
const allAlbumTracks = [];
for (const album of artistAlbums) {
  const tracks = await site.releases.query({ 'metadata.structureId': album.id });
  allAlbumTracks.push(...tracks);
}
```

## Key Design Principles

1. **Completely Generic**: No hardcoded content types or relationships
2. **Arbitrary Hierarchies**: Any structure can be parent/child of any other
3. **Efficient P2P Queries**: Designed for optimal PeerBit query performance
4. **Optional Relationships**: All relationships are optional - structures can be standalone
5. **Flexible Metadata**: Content can reference structures however makes sense

## Common Patterns

### Standalone Content
Content doesn't need to belong to any structure:
```typescript
// A standalone documentary
{
  title: "Free Culture Documentary",
  // no metadata.structureId - completely independent
}
```

### Multiple Structure References
Content can reference multiple structures:
```typescript
// A song that's part of an album AND a compilation
{
  title: "Bohemian Rhapsody",
  metadata: {
    structureId: albumId,        // Part of "A Night at the Opera"
    compilationIds: [comp1, comp2] // Also in various compilations
  }
}
```

### Cross-Category Structures
Structures can span different content categories:
```typescript
// A director structure containing both movies and TV shows
Director: "Christopher Nolan" (type: 'director')
├── Movie: "Inception" (categorySlug: 'movies', metadata.structureId: nolan-id)
├── Movie: "Interstellar" (categorySlug: 'movies', metadata.structureId: nolan-id)
└── TV Show: "Westworld" (categorySlug: 'tv-shows', metadata.structureId: nolan-id)
```