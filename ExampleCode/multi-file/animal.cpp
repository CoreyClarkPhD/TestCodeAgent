#include <iostream>

class Animal {
public:
  void speak() {
    std::cout << "hey im a animal" << std::endl;
  }
};

class Dog : public Animal {
public:
  void speak() {
    std::cout << "hey im a dog" << std::endl;
  }
};

class Cat : public Animal {
public:
  void speak() {
    std::cout << "hey im a cat" << std::endl;
  }
};
