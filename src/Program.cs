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
            string text = "0x12345";
            Console.WriteLine($"{text}");
            var tokens = lex.Parse(text);
            sw.Stop();
            
            Console.WriteLine($"Time elapsed {sw.Elapsed}");
            for(int i = 0; i < tokens.Length; i++)
                Console.WriteLine("{0} {1}", text[i], tokens[i]);
        }
    }
}
