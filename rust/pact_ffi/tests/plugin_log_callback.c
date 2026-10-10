#include "pact.h"

#ifdef __cplusplus
extern "C"
#endif
void plugin_log_callback(const char *plugin_instance_id,
                         const char *test_run_id,
                         const char *level,
                         const char *target,
                         const char *message) {
  (void)plugin_instance_id;
  (void)test_run_id;
  (void)level;
  (void)target;
  (void)message;
}

void check_plugin_log_callback(void) {
  pactffi_register_plugin_log_callback(plugin_log_callback);
  pactffi_register_plugin_log_callback(NULL);
}
