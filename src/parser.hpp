typedef int Operator;

struct Expression {
	char type;
	int value;
	struct Expression *left, *right;
	Operator op;
};

typedef Expression ASTNode;
extern int parseProgram(ASTNode **);