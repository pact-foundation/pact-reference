#include <pact-cpp.h>

int main(int argc, char *argv[]) {
    int port = pactffi_create_mock_server("{}", "127.0.0.1:0", false);
    pactffi_mock_server_matched(port);
    return 0;
}
