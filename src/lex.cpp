#include "lex.hpp"

#include <stdio.h>

bool isWhitespace(int ch) {
	switch (ch) {
		case '\n':
		case ' ':
		case '\t':
			return true;
			break;
	}
	return false;
}

tokenType token;

void getNextToken() {
	int ch;

	do {
		ch = getchar();
		if (ch < 0) {
			token.classType = EoF;
			token.repr = '#';
			return;
		}
	} while (isWhitespace(ch));

	if ('0' <= ch && ch <= '9')
		token.classType = DIGIT;
	else
		token.classType = ch;

	token.repr = ch;
}