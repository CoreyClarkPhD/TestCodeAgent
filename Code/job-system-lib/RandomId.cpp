#include "RandomId.h"
#include <iostream>
#include <random>
#include <ctime>
#include <string>

std::string generateRandomID(int length) {
    static const char charset[] =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    static const int charsetSize = sizeof(charset) - 1;
    static std::mt19937 generator(static_cast<unsigned>(std::time(0)));
    static std::uniform_int_distribution<int> distribution(0, charsetSize - 1);

    std::string randomID;
    for (int i = 0; i < length; ++i) {
        randomID += charset[distribution(generator)];
    }
    return randomID;
}
