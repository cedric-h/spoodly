using System;
using System.Collections.Generic;

namespace spoodly
{
    using DictionaryType = Dictionary<(char c, int state), (int state, LexerToken token)>;

    public static class LutUtils
    {
        public static void PrintLut(DictionaryType Lut)
        {
            foreach(var kv in Lut)
            {
                Console.WriteLine($"({kv.Key.c}, {kv.Key.state}) -> ({kv.Value.state}, {kv.Value.token})");
            }
        }
    }
}