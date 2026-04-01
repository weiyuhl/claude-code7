#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef void (*StreamCallback)(const char*, void*);

void *create_session(const char *config_str);

char *send_message(void *session_ptr, const char *content);

int stream_message(void *session_ptr,
                   const char *content,
                   StreamCallback callback,
                   void *user_data);

void destroy_session(void *session_ptr);

char *get_messages(void *session_ptr);

void free_string(char *s);

bool set_provider(void *session_ptr, const char *provider_name, const char *api_key);

char *list_models(void *session_ptr);

char *get_balance(void *session_ptr);
