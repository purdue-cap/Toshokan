#include <cmath>

int ANONYMOUS__sqrt_impl(int i) {
    double result = sqrt(i);
    if (result != result) {
        return -1;
    }
    return (int)result;
}