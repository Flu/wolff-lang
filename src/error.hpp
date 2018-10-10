#include <iostream>

void Error(const char* errorText) {
	if (errorText)
		std::cerr << errorText;
}