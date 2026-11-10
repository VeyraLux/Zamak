// External scanner for Zamak: line ends and indentation.
//
// Like the compiler's lexer, it ignores blank lines and lines that hold only a
// comment, and it never ends a line inside ( ), [ ] or { }: the grammar doesn't ask
// for a line end there, so the scanner leaves the line break as whitespace.
// All three tokens are zero-width; the whitespace after them is skipped as extras.

#include "tree_sitter/alloc.h"
#include "tree_sitter/array.h"
#include "tree_sitter/parser.h"

#include <stdint.h>
#include <string.h>

enum TokenType { NEWLINE, INDENT, DEDENT, ERROR_SENTINEL };

typedef struct {
  Array(uint16_t) indents;
} Scanner;

static void skip(TSLexer *lexer) { lexer->advance(lexer, true); }

void *tree_sitter_zamak_external_scanner_create(void) {
  Scanner *scanner = ts_calloc(1, sizeof(Scanner));
  array_init(&scanner->indents);
  array_push(&scanner->indents, 0);
  return scanner;
}

void tree_sitter_zamak_external_scanner_destroy(void *payload) {
  Scanner *scanner = (Scanner *)payload;
  array_delete(&scanner->indents);
  ts_free(scanner);
}

unsigned tree_sitter_zamak_external_scanner_serialize(void *payload, char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  unsigned size = 0;
  // The bottom level is always 0, so it isn't stored.
  for (uint32_t i = 1; i < scanner->indents.size; i++) {
    if (size + sizeof(uint16_t) > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) break;
    memcpy(buffer + size, &scanner->indents.contents[i], sizeof(uint16_t));
    size += sizeof(uint16_t);
  }
  return size;
}

void tree_sitter_zamak_external_scanner_deserialize(void *payload, const char *buffer,
                                                    unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  array_clear(&scanner->indents);
  array_push(&scanner->indents, 0);
  for (unsigned i = 0; i + sizeof(uint16_t) <= length; i += sizeof(uint16_t)) {
    uint16_t indent;
    memcpy(&indent, buffer + i, sizeof(uint16_t));
    array_push(&scanner->indents, indent);
  }
}

bool tree_sitter_zamak_external_scanner_scan(void *payload, TSLexer *lexer,
                                             const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  // During error recovery every symbol is valid. Indentation still opens and closes
  // blocks, so one bad line doesn't swallow the rest of the file, but no line end is
  // invented.
  bool error_recovery = valid_symbols[ERROR_SENTINEL];

  lexer->mark_end(lexer);

  bool found_end_of_line = false;
  uint32_t indent = 0;
  for (;;) {
    int32_t c = lexer->lookahead;
    if (c == '\n') {
      found_end_of_line = true;
      indent = 0;
      skip(lexer);
    } else if (c == ' ') {
      indent++;
      skip(lexer);
    } else if (c == '\t') {
      // The compiler rejects tabs in indentation; `zam fmt` expands them to 4 columns.
      indent += 4;
      skip(lexer);
    } else if (c == '\r' || c == '\f') {
      indent = 0;
      skip(lexer);
    } else if (c == '#') {
      // A comment after code on the same line: let the comment token take it first.
      if (!found_end_of_line) return false;
      // A line that holds only a comment doesn't count for indentation.
      while (lexer->lookahead && lexer->lookahead != '\n') skip(lexer);
    } else if (lexer->eof(lexer)) {
      indent = 0;
      found_end_of_line = true;
      break;
    } else {
      break;
    }
  }

  if (!found_end_of_line) return false;

  uint16_t current = *array_back(&scanner->indents);
  if (valid_symbols[INDENT] && indent > current) {
    array_push(&scanner->indents, (uint16_t)indent);
    lexer->result_symbol = INDENT;
    return true;
  }
  if (valid_symbols[DEDENT] && indent < current && scanner->indents.size > 1) {
    array_pop(&scanner->indents);
    lexer->result_symbol = DEDENT;
    return true;
  }
  if (valid_symbols[NEWLINE] && !error_recovery) {
    lexer->result_symbol = NEWLINE;
    return true;
  }
  return false;
}
