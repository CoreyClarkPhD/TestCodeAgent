#include <iostream>
#include <vector> // Include vector header

struct Contact {
    std::string name;
    std::string address;
    std::string phoneNumber;
    Contact* knows; // Pointer to another contact that this contact knows
};

int main() {
    std::vector<Contact> contacts;
    while (true) {
        Contact newContact; // Add a semicolon here
        std::cout << "Enter contact name (or 'exit' to finish): ";
        std::getline(std::cin, newContact.name);
        if (newContact.name == "exit") {
            break;
        }
        std::cout << "Enter contact address: ";
        std::getline(std::cin, newContact.address);
        std::cout << "Enter contact phone number: ";
        std::getline(std::cin, newContact.phoneNumber);

        std::string knowsName;
        std::cout << "Enter the name of someone this contact knows (or 'none'): ";
        std::getline(std::cin, knowsName);
        if (knowsName != "none") {
            for (auto& contact : contacts) { // Changed from auto to auto& to be able to take the address properly
                if (contact.name == knowsName) {
                    newContact.knows = &contact;
                    break;
                }
            }
        } else {
            newContact.knows = nullptr; // Fix typo from 'kows' to 'knows'
        }

        contacts.push_back(newContact);
    }

    std::cout << "\nContact List:\n";
    for (const auto& contact : contacts) {
        std::cout << "Name: " << contact.name << "\nAddress: " << contact.address
                  << "\nPhone Number: " << contact.phoneNumber;
        if (contact.knows != nullptr) {
            std::cout << "\nKnows: " << contact.knows->name;
        }
        std::cout << "\n\n";
    }

    return 0;
}
