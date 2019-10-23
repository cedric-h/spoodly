using System;
using System.Diagnostics;

namespace spoodly
{
    class Program
    {
        static void Main(string[] args)
        {
            Stopwatch sw = new Stopwatch();
            sw.Start();
            var lex = new DynamicLexer();
            sw.Stop();
            Console.WriteLine($"Time elapsed {sw.Elapsed}");
            //lex.PrintLut();

            while(true)
            {
                sw.Reset();
                try
                {
                    Console.Write(">>> ");
                    string text = Console.ReadLine();
                    sw.Start();
                    var tokens = lex.Parse(text);
                    sw.Stop();
                    Console.WriteLine($"Time elapsed {sw.Elapsed}");
                    for(int i = 0; i < tokens.Length; i++)
                        Console.WriteLine("{0} {1}", text[i], tokens[i]);
                }
                catch(UnknownLexerStateException le)
                {
                    Console.WriteLine("Uh-oh. Our lexer has encountered an unexpected error.\n" + le.Message);
                }
            }
        }
    }
}
