#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>

typedef struct {} Tokenizer;
typedef struct {} TokenError;
int32_t toto_tokenizer_new(const char* source, Tokenizer** tokenizer);
int32_t toto_tokenizer_next(Tokenizer* tokenizer, int32_t* token_type,
    int32_t* has_text, const char** text, size_t* len, int32_t* has_error,
    TokenError** error);
int32_t toto_tokenizer_destroy(Tokenizer* tokenizer);
int32_t toto_error_explain(TokenError* error, const char* source);
int32_t toto_error_destroy(TokenError* error);

int main(int argc, const char** argv) {
    const char* source = ""
    "[package]\n"
    "authors = [\"Machtan <jako3047@gmail.com>\"]\n"
    "name = \"space-toml-clib\"\n"
    "version = \"0.1.0\"\n"
    "\n"
    "[dependencies]\n"
    "libc = \"0.2.15\"\n"
    "\n"
    "[dependencies.space-toml]\n"
    "path = \"../space-toml\"\n"
    "\n"
    "[lib]\n"
    "crate-type = [\"cdylib\"]\n"
    "name = \"toto\"\n"
    "";
    Tokenizer* tokens = NULL;
    int32_t create_err = toto_tokenizer_new(source, &tokens);
    if (create_err != 0) {
        printf("Error creating tokenizer!\n");
        return 1;
    } else {
        puts("Created!");
    }
    
    int32_t token_type;
    int32_t has_text;
    const char* text;
    size_t len;
    int32_t has_error;
    TokenError* error;
    int32_t has_token;
    int i = 0;
    while (toto_tokenizer_next(tokens, &token_type, &has_text, &text, &len, &has_error, &error) == 0) {
        if (has_text) {
            printf("%04" PRId32 ": (%02" PRId32 ") %p [%zu]\n", i, token_type, text, len);
            printf("%.*s\n", len, text);
        } else {
            printf("%04" PRId32 ": (%02" PRId32 ")\n", i, token_type);
        }
        
        i += 1;
    }
    if (has_error) {
        printf("Found error while parsing!\n");
        toto_error_explain(error, source);
        toto_error_destroy(error);
    }
    
    int32_t destroy_err = toto_tokenizer_destroy(tokens);
    if (destroy_err != 0) {
        printf("Error destroying tokenizer!\n");
        return 1;
    }
    return 0;
}