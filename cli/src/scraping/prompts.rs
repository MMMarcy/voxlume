use serde_json::json;
pub const PARSE_HTML_INSTRUCTIONS: &str = r"
You are a web scraping assistant. Your task is to extract information
about new audiobook releases from the provided HTML content.

HTML content to parse:

```html
{html}
```
";

pub const CREATE_DESCRIPTION_FOR_EMBEDDING: &str = r"
You are a search optimization specialist. Your task is to rewrite the
provided audiobook description to improve its findability for
vector-based searches.

The rewritten text should contain keywords and phrases that a user
might use in a generic query for this type of content. The goal is to
optimize the description for generating effective embeddings.

Here is the original description:
```
{description}
```

Only ouput the embeddable description without any preamble.
";

pub const CREATE_VERY_SHORT_DESCRIPTION_PROMPT: &str = r"
Create a very short summary of at most a couple of concise sentence
summarizing the audiobook description. This description should be suitable for a
brief overview.

Description:
```
{description}
```

Only output the description without any preamble.
";

pub fn get_submission_list_schema(base_path: &String) -> serde_json::Value {
    let tmp = format!(
        "The fully qualified URL to the audiobook's details page. Use {base_path} as the domain."
    );
    json!({
            "type": "object",
            "properties": {
                "submissions": {
                    "type": "array",
                    "description": "A list of new audiobook submissions extracted from the source.",
                    "items": {
                        "type": "object",
                        "description": "Represents a newly submitted audiobook entry found on the main page.",
                        "properties": {
                            "submission_date": {
                                "type": "string",
                                "description": "The date of submission, ideally in YYYY-MM-DD format."
                            },
                            "url": {
                                "type": "string",
                                "description": tmp
                            }
                        },
                        "required": [
                            "submission_date",
                            "url"
                        ]
                    }
                }
            },
            "required": [
                "submissions"
            ]
    })
}

pub fn get_audiobook_schema() -> serde_json::Value {
    json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "The full title of the audiobook."
                },
                "categories": {
                    "type": "array",
                    "description": "A list of categories or genres associated with the audiobook. e.g., 'Fiction', 'Science Fiction'.",
                    "items": {
                        "type": "string"
                    }
                },
                "language": {
                    "type": "string",
                    "description": "The language the audiobook is narrated in."
                },
                "keywords": {
                    "type": "array",
                    "description": "A list of keywords or tags related to the audiobook's content or themes.",
                    "items": {
                        "type": "string"
                    }
                },
                "cover_url": {
                    "type": "string",
                    "description": "The direct, fully qualified URL to the audiobook's cover image."
                },
                "authors": {
                    "type": "array",
                    "description": "The names of the authors who wrote the book.",
                    "items": {
                        "type": "string"
                    }
                },
                "read_by": {
                    "type": "array",
                    "description": "The names of the narrators or voice actors.",
                    "items": {
                        "type": "string"
                    }
                },
                "format": {
                    "type": "string",
                    "description": "The audio file format, e.g., 'MP3', 'M4B'."
                },
                "bitrate": {
                    "type": "string",
                    "description": "The bitrate of the audio file, if available. e.g., '128kbps'."
                },
                "unabridged": {
                    "type": "boolean",
                    "description": "Set to true if the audiobook is an unabridged version."
                },
                "description": {
                    "type": "string",
                    "description": "The full HTML content of the audiobook's description section. Remove any other metadata that was already parsed as part by other fields from the description."
                },
                "file_size": {
                    "type": "string",
                    "description": "The total file size of the audiobook, if specified. e.g., '1.2 GB'."
                },
                "runtime": {
                    "type": "string",
                    "description": "The total runtime of the audiobook, preferably in HH:MM:SS format."
                },
                "is_part_of_series": {
                    "type": "boolean",
                    "description": "Set to true if the audiobook is part of a series."
                },
                "series": {
                    "type": "string",
                    "description": "The title of the series the audiobook belongs to. Extract this even if it's part of the main title."
                },
                "series_volume": {
                    "type": "string",
                    "description": "The volume or book number within the series. e.g., 'Book 1', 'Volume 3'."
                },
                "upload_date": {
                    "type": "string",
                    "description": "The date of the upload in the format YYYY-MM-DD. E.g. 2025-09-18."
                }
            },
            "required": [
                "title",
                "categories",
                "language",
                "keywords",
                "cover_url",
                "authors",
                "read_by",
                "format",
                "unabridged", // Even with a default, it's not nullable, so it's required.
                "description"
            ]

    })
}
