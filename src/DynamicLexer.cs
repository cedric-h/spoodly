using System;
using System.Linq;
using System.Collections.Generic;

namespace spoodly
{
    // Cursed code lol
    public class LexerRule<T>
    {
        public enum TransitionStates
        {
            Next,
            Self,
            Previous
        }
        public static char[] AnyChar = Enumerable.Range(0, char.MaxValue+1).Select(i => (char) i).Where(c => !char.IsControl(c)).ToArray();
        //private int stateCounter = 0;
        private struct StateStruct
        {
            public char[] arr;
            public int i;
            public T token;
        }
        private static Dictionary<char, int> MaxStates = new Dictionary<char, int>();
        private Dictionary<(char c, int state), (int state, T token)> LutDictionary;
        private Stack<StateStruct> ActionStack;
        //private StateStruct currentState;
        private int currentStateIndex;
        private int stateStateIndex;

        public LexerRule(Dictionary<(char c, int state), (int state, T token)> dictionary)
        {
            LutDictionary = dictionary;
            ActionStack = new Stack<StateStruct>();
            currentStateIndex = 0;
            stateStateIndex = 0;
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
                i = currentStateIndex,
                token = token
            });
            return this;
        }

        // Keeps the current state
        public LexerRule<T> MatchAny(string c, T token) => MatchAny(c.ToCharArray(), token);
        // Keeps the current state
        public LexerRule<T> MatchAny(char[] c, T token)
        {
            DoTransition(c, currentStateIndex, currentStateIndex, token);
            return this;
        }

        // Applies all current states
        public LexerRule<T> Then(Action<LexerRule<T>> a)
        {
            currentStateIndex++;
            while(ActionStack.Count > 0)
            {
                var st = ActionStack.Pop();
                DoTransition(st.arr, st.i, currentStateIndex, st.token);
            }
            a.Invoke(this);
            currentStateIndex--;
            return this;
        }

        // References previous state
        public LexerRule<T> Until(char c, T token) => Until(new char[] { c }, token);
        // References previous state
        public LexerRule<T> Until(string c, T token) => Until(c.ToCharArray(), token);
        // References previous state
        public LexerRule<T> Until(char[] c, T token)
        {
            DoTransition(c, currentStateIndex, currentStateIndex-1, token);
            return this;
        }

        private void SafeSetLut(char c, int from, int to, T token)
        {
            int v = 0;
            if(MaxStates.TryGetValue(c, out v))
            {
                v++;
                MaxStates[c] = Math.Max(v, from);
                LutDictionary[(c, v)] = (to, token);
            }
        }

        private void DoTransition(char[] c, int from, int to, T token)
        {
            if(c == null)
            {
                LutDictionary[((char) 0xFF, from)] = (to, token);
                return;
            }
            foreach(char x in c)
            {
                LutDictionary[(x, from)] = (to, token);
            }
        }
    }

    public class DynamicLexer
    {
        private Dictionary<(char c, int state), (int state, LexerToken token)> Lut;
        public DynamicLexer()
        {
            Lut = new Dictionary<(char c, int state), (int state, LexerToken token)>();
            new LexerRule<LexerToken>(Lut).MatchAny((char[]) null, LexerToken.Unknown);
            new LexerRule<LexerToken>(Lut)
                .Match('"', LexerToken.StringLiteral)
            .Then((t1) =>
                t1.MatchAny((char[]) null, LexerToken.StringLiteral)
                    .Until('"', LexerToken.StringLiteral)
                    .Match('\\', LexerToken.StringLiteral)
                .Then((t2) =>
                    t2.Until((char[]) null, LexerToken.StringLiteral)
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