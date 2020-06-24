#include <cmath>

int sqrt_impl(int i) {
    double result = sqrt(i);
    if (result != result) {
        return -1;
    }
    return (int)result;
}