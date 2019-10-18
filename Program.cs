using System;
using System.Diagnostics;

namespace spoodly
{
    class Program
    {
        static void Main(string[] args)
        {
            var lex = new Lexer();
            string text = "\"1\\\" 6.7e-11\"6.7e-11";
            Console.WriteLine($"{text}");

            Stopwatch sw = new Stopwatch();
            sw.Start();
            var tokens = lex.Parse(text);
            sw.Stop();
            
            Console.WriteLine($"Time elapsed {sw.Elapsed}");
            for(int i = 0; i < tokens.Length; i++)
                Console.WriteLine("{0} {1}", text[i], tokens[i]);
        }
    }
}
