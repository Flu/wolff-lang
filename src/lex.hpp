#define EoF 256
#define DIGIT 257

struct tokenType {
	int classType;
	char repr;
};

extern tokenType token;
extern void getNextToken();