+++
id = "peas-tbl6t"
title = "Pagination with page dots instead of scrolling"
type = "feature"
status = "todo"
priority = "normal"
parent = "peas-7z7f5"
created = "2026-01-18T19:30:03.886585200Z"
updated = "2026-01-18T19:30:03.886585200Z"
+++

Replace scrolling list with pagination and page indicator dots.

Current: List scrolls when cursor moves past visible area
New: Show one page of items at a time with dot indicators at bottom

Implementation:
- Calculate items per page based on available height
- Show page indicator dots (••••) where each dot represents a page
- Highlight current page dot (e.g., make it brighter or different color)
- Page up/down with PageUp/PageDown keys
- j/k still move within page, wrapping to next/prev page at boundaries

Example with 3 pages, on page 1:
```
 •••
```
Where the first dot is highlighted/bright.
