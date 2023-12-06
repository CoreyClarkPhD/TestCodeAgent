#include <iostream>
#include "./animal.cpp"

int main() {
    Dog *dog = new Dog();

    // Speak twice
    dog->speak();
    dog->speak(); // Corrected from speek() to speak()

    return 0;
}
