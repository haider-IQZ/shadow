#include <stdio.h>

int main(void) {
    return puts("Hello from Shadow!") < 0 ? 1 : 0;
}
