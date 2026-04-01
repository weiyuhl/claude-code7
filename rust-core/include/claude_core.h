#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef void (*StreamCallback)(const char*, void*);

void *claude_create_session(const char *config_json);

char *claude_send_message(void *session, const char *content);

int claude_stream_message(void *session,
                          const char *content,
                          StreamCallback callback,
                          void *user_data);

void claude_destroy_session(void *session);

char *claude_get_messages(void *session);

char *claude_list_models(void *session);

char *claude_get_balance(void *session);

void claude_free_string(char *s);

void *create_session(const char *config_str);

char *send_message(void *session_ptr, const char *content);

char *stream_message(void *session_ptr,
                     const char *content,
                     void (*_callback)(const char*, void*),
                     void *_user_data);

void destroy_session(void *session_ptr);

char *get_messages(void *session_ptr);

void free_string(char *ptr);

bool set_provider(void *session_ptr, const char *provider_name, const char *api_key);

char *list_models(void *session_ptr);

char *get_balance(void *session_ptr);
