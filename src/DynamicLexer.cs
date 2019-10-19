using System;
using System.Linq;
using System.Collections.Generic;

namespace spoodly
{
    // Cursed code lol
    public class LexerRule
    {
        public static char[] AnyChar = new char[] { (char) 0xFF };
    }

    public class LexerRule<T>
    {
        private struct StateStruct
        {
            public char[] arr;
            public int i;
            public T token;
        }

        private static int totalStates = 1;
        private Dictionary<(char c, int state), (int state, T token)> LutDictionary;
        private Stack<StateStruct> ActionStack;
        private Stack<int> StateStack;

        public LexerRule(Dictionary<(char c, int state), (int state, T token)> dictionary)
        {
            LutDictionary = dictionary;
            ActionStack = new Stack<StateStruct>();
            StateStack = new Stack<int>();
            StateStack.Push(0); // Default state is 0
        }
        
        // Pushes the current state
        public LexerRule<T> Match(char c, T token) => Match(new char[] { c }, token);
        // Pushes the current state
        public LexerRule<T> Match(string c, T token) => Match(c.ToCharArray(), token);
        // Pushes the current state
        public LexerRule<T> Match(char[] c, T token)
        {
            ActionStack.Push(new StateStruct()
            {
                arr = c,
                i = GetCurrentState(),
                token = token
            });
            return this;
        }

        // Keeps the current state
        public LexerRule<T> MatchAny(string c, T token) => MatchAny(c.ToCharArray(), token);
        // Keeps the current state
        public LexerRule<T> MatchAny(char[] c, T token)
        {
            DoTransition(c, GetCurrentState(), GetCurrentState(), token);
            return this;
        }

        // References previous state
        public LexerRule<T> Until(char c, T token) => Until(new char[] { c }, token);
        // References previous state
        public LexerRule<T> Until(string c, T token) => Until(c.ToCharArray(), token);
        // References previous state
        public LexerRule<T> Until(char[] c, T token)
        {
            DoTransition(c, GetCurrentState(), GetPreviousState(), token);
            return this;
        }

        // Applies all current states
        public LexerRule<T> Then(Action<LexerRule<T>> a) => Then(1, a);

        // Applies all current states
        public LexerRule<T> Then(int n, Action<LexerRule<T>> a)
        {
            int next = GetNextState();
            StateStack.Push(next);

            while(ActionStack.Count > 0)
            {
                var ak = ActionStack.Pop();
                DoTransition(ak.arr, ak.i, next, ak.token);
            }

            a?.Invoke(this);
            StateStack.Pop();
            return this;
        }

        private void DoTransition(char[] c, int from, int to, T token)
        {
            foreach(char x in c)
                LutDictionary[(x, from)] = (to, token);
        }

        private int GetPreviousState()
        {
            var cur = StateStack.Pop();
            var prv = StateStack.Peek();
            StateStack.Push(cur);
            return prv;
        }

        private int GetCurrentState() => StateStack.Peek();
        private int GetNextState() => totalStates++;
    }

    public class DynamicLexer
    {
        private Dictionary<(char c, int state), (int state, LexerToken token)> Lut;
        public DynamicLexer()
        {
            Lut = new Dictionary<(char c, int state), (int state, LexerToken token)>();
            new LexerRule<LexerToken>(Lut).MatchAny(LexerRule.AnyChar, LexerToken.Unknown);
            new LexerRule<LexerToken>(Lut)
                .Match('"', LexerToken.StringLiteral)
            .Then((t1) =>
                t1.MatchAny(LexerRule.AnyChar, LexerToken.StringLiteral)
                    .Until('"', LexerToken.StringLiteral)
                    .Match('\\', LexerToken.StringLiteral)
                .Then((t2) =>
                    t2.Until(LexerRule.AnyChar, LexerToken.StringLiteral)
                )
            );
            new LexerRule<LexerToken>(Lut)
                .Match('0', LexerToken.IntegerLiteral)
            .Then((t1) =>
                t1.Match("xX", LexerToken.HexSeparator).Then((t2) =>
                    t2.MatchAny("0123456789ABCDEFabcdef", LexerToken.IntegerHexLiteral)
                ).Match("bB", LexerToken.BinarySeparator).Then((t2) =>
                    t2.MatchAny("01", LexerToken.IntegerBinaryLiteral)
                ).Match("1234567890", LexerToken.IntegerLiteral)
            ).Match("123456789", LexerToken.IntegerLiteral).Then((t1) =>
                t1.Match("Ee", LexerToken.ExponentIdentifier).Then((t2) =>
                    t2.Match("+-", LexerToken.ExponentSign).Then((t3) =>
                        t3.MatchAny("1234567890", LexerToken.IntegerLiteral)
                    ).Match("1234567890", LexerToken.IntegerLiteral).Then((t3) =>
                        t3.MatchAny("1234567890", LexerToken.IntegerLiteral)
                    )
                )
            );
        }

        public LexerToken[] Parse(string text)
        {
            int currentState = 0;
            LexerToken[] states = new LexerToken[text.Length];
            for(int i = 0; i < text.Length; i++)
            {
                (int state, LexerToken token) res;
                // We take advantage of C# operator short circuiting
                if(Lut.TryGetValue((text[i], currentState), out res) || Lut.TryGetValue(((char) 0xFF, currentState), out res))
                {
                    currentState = res.state;
                    states[i] = res.token;
                }
                else
                {
                    throw new LexerException($"Lexer was put in an undefined state: (Column: {i}, '{text[i]}', {currentState})");
                }
            }
            return states;
        }
    }
}