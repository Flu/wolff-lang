#include "parser.h"
#include "backend.h"
#include "error.h"

#include <string>
#include <vector>

int processInput() {
	string expr;
	while (true) {
		cout << ":>";
		getline(cin, expr);
		if (expr == "exit")
			return 0;
		
	}
	return 1;
}

void startInteractiveShell() {
	cout << "Dharma interpretor (v0.1 alpha)\n";
	cout << "by Fluturel Adrian, 2018\n";
	if (processInput() == 0)
		exit(0); // Program terminated with success
}

void parseOptions(int argc, char** argv) {
	if (argc < 2)
		exit(1); // Not enough arguments
	if (argv[1] == "-i" || argv[1] == "--interactive")
		startInteractiveShell();
}

int main(int argc, char *argv[]) {
	//parseOptions(argc, argv);
	AST *tree;
	if (!parseProgram(&tree))
		throw "No top-level expression";
	
	Process(tree);
	return 0;
}