#pragma once
#include "RandomId.h"
#include <string>
class Job {
  public:
    Job(std::string type, std::string input){
        this->type = type;
        this->input = input;
        id = generateRandomID(10);
    }

    // Copy constructor with new id
    Job(const Job &other) {
        this->type = other.type;
        this->input = other.input;
        id = generateRandomID(10);
    }

    std::string input = "";
    std::string type = "";
    std::string id = "";
};
