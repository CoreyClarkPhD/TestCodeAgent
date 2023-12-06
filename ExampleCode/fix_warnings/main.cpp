#include <iostream>

int main() {
    int x;
    std::cout << "Enter a number: ";
    std::cin >> x;

    int y;
    std::cout << "The sum of " << x << " and " << y << " is: " << x + y << std::endl;

    for (unsigned int i = 0; i >= 0; --i) {
        std::cout << i << std::endl;
    }

    double pi = 3.14159;
    std::cout << "Value of pi: %f" << pi << std::endl;

    return 0;
}

