#include "lex.hpp"
#include "error.hpp"
#include "parser.hpp"

#include <stdlib.h>

Expression *newExpression() {
	return new Expression;
}

void freeExpression(Expression *expr) {
	delete[] expr;
}

int parseOperator(Operator *oper_p);
int parseExpression(Expression **expr_p);

int parseOperator(Operator *op) {
	if (token.classType == '+') {
		*op = '+';
		getNextToken();
		return 1;
	}
	if (token.classType == '*') {
		*op = '*';
		getNextToken();
		return 1;
	}
	return 0;
}

int parseExpression(Expression **expr_p) {
	Expression *expr = *expr_p = newExpression();

	if (token.classType = DIGIT) {
		expr->type = 'D';
		expr->value = token.repr - '0';
		getNextToken();
		return 1;
	}

	if (token.classType == '(') {
		expr->type = 'P';
		getNextToken();
		if (!parseExpression(&expr->left))
			Error("Missing expression");
		if (!parseOperator(&expr->op))
			Error("Missing operator");
		if (!parseExpression(&expr->right))
			Error("Missing expression");
		if (token.classType != ')')
			Error("Missing right paranthesis");
		getNextToken();
		return 1;
	}
	freeExpression(expr);
	return 0;
}