#include <stdio.h>

int loopy()
{
	int count = 0;
	while (count < 10) {
		count = count + 1;
	}
	return count;
}

int main()
{
	printf("Hello, world!");
	if (1ul + 2ul == 3ul)
	{
		printf("3");
	}
	int loopres = loopy();
	printf("%d", loopres);
	return 0;
}
