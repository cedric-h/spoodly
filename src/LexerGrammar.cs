using System;
using System.Linq;
using System.Collections.Generic;

namespace spoodly
{
    public class LexerRule
    {
        public static readonly char[] AnyChar = new char[] { (char)0xFF };
        public static readonly string LowerAlpha = "abcdefghijklmnopqrstuvwxyz";
        public static readonly string UpperAlpha = "abcdefghijklmnopqrstuvwxyz".ToUpper();
        public static readonly string Numerics = "0123456789";
    }

    /// <summary>
    /// Metprogramming interface for generating LUTs
    /// </summary>
    /// <typeparam name="T">Token Type</typeparam>
    public class LexerGrammar<T>
    {
        private struct TransitionStruct
        {
            public char c;
            public int from;
            public int to;
            public string target;
            public T token;

            public TransitionStruct(char p1, int p2, int p3, string p4, T p5)
            {
                this.c = p1;
                this.from = p2;
                this.to = p3;
                this.target = p4;
                this.token = p5;
            }
        }

        private Dictionary<(char c, int state), (int state, T token)> LutDictionary;
        private Dictionary<string, Action<LexerRule>> Actions;
        private Dictionary<string, int> StateNames;
        private List<TransitionStruct> Transitions;

        public int TotalStates 
        {
            get; private set;
        }

        public LexerGrammar(Dictionary<(char c, int state), (int state, T token)> Lut)
        {
            this.LutDictionary = Lut;
            this.Transitions = new List<TransitionStruct>();
            this.StateNames = new Dictionary<string, int>();
            this.Actions = new Dictionary<string, Action<LexerRule>>();
            this.TotalStates = 1;
        }

        public LexerRule AddNewRule() => new LexerRule(this);

        public void AddNewAction(string n, Action<LexerRule> a) => Actions[n] = a;

        public void ConstructLut()
        {
            foreach(var t in Transitions)
            {
                if(t.target == null || t.target == "")
                    LutDictionary[(t.c, t.from)] = (t.to, t.token);
                else
                    LutDictionary[(t.c, t.from)] = (StateNames[t.target], t.token);
            }
        }

        /// <summary>
        /// A rule that is part of the grammar. Note that the LUT
        /// is not directly updated here, and is instead queued
        /// through a TransitionStruct on the Transitions list.
        /// </summary>
        public class LexerRule
        {
            private struct StateStruct
            {
                public char ch;
                public int from;
                public T token;
            }

            private Stack<int> StateStack;
            private Stack<StateStruct> ActionStack;
            private LexerGrammar<T> Parent;

            internal LexerRule(LexerGrammar<T> p)
            {
                Parent = p;
                ActionStack = new Stack<StateStruct>();
                StateStack = new Stack<int>();
                StateStack.Push(0); // Default state is always 0
            }

            #region Match

            // Pushes the current state
            public LexerRule Match(string c, T token) => Match(c.ToCharArray(), token);
            // Pushes the current state
            public LexerRule Match(char[] c, T token)
            {
                foreach (var ch in c) Match(ch, token);
                return this;
            }
            // Pushes the current state
            public LexerRule Match(char c, T token)
            {
                ActionStack.Push(new StateStruct()
                {
                    ch = c,
                    from = GetCurrentState(),
                    token = token
                });
                return this;
            }

            #endregion

            #region MatchAny

            // Keeps the current state
            public LexerRule MatchAny(string c, T token) => MatchAny(c.ToCharArray(), token);
            // Keeps the current state
            public LexerRule MatchAny(char[] c, T token)
            {
                foreach(var ch in c) MatchAny(ch, token);
                return this;
            }
            // Keeps the current state
            public LexerRule MatchAny(char c, T token)
            {
                Parent.Transitions.Add(new TransitionStruct(c, GetCurrentState(), GetCurrentState(), null, token));
                return this;
            }

            #endregion
            
            #region MatchString

            public LexerRule MatchString(string s, T defaultToken, T token)
            {
                var cur = GetCurrentState();
                for(int i = 0; i < s.Length-1; i++)
                {
                    int next = GetNextState();
                    if(i > 0)
                    Parent.Transitions.Add(new TransitionStruct((char) 0xFF, GetCurrentState(), 0, null, defaultToken));
                    Parent.Transitions.Add(new TransitionStruct(s[i], GetCurrentState(), next, null, defaultToken));
                    StateStack.Push(next);
                }
                Parent.Transitions.Add(new TransitionStruct(s[s.Length-1], GetCurrentState(), 0, null, token));
                //Parent.Transitions.Add(new TransitionStruct((char) 0xFF, GetCurrentState(), 0, null, defaultToken));
                StateStack.Push(cur); // TODO: Should I pop instead? Yes.
                return this;
            }

            #endregion

            #region Until

            // References previous state
            public LexerRule Until(string c, T token) => Until(c.ToCharArray(), token);
            // References previous state
            public LexerRule Until(char[] c, T token)
            {
                foreach(var ch in c) Until(ch, token);
                return this;
            }
            // References previous state
            public LexerRule Until(char c, T token)
            {
                Parent.Transitions.Add(new TransitionStruct(c, GetCurrentState(), GetPreviousState(), null, token));
                return this;
            }

            #endregion

            #region Then
            
            // Applies all current states
            public LexerRule Then(Action<LexerRule> a)
            {
                int next = GetNextState();
                
                while(ActionStack.Count > 0)
                {
                    var ak = ActionStack.Pop();
                    Parent.Transitions.Add(new TransitionStruct(ak.ch, ak.from, next, null, ak.token));
                }

                StateStack.Push(next);
                a?.Invoke(this);
                StateStack.Pop();
                return this;
            }

            #endregion

            #region Misc Methods

            public LexerRule Name(string n) 
            {
                Parent.StateNames[n] = GetCurrentState();
                return this;
            }

            public LexerRule GotoWhen(string n, Action<LexerRule> a)
            {
                Stack<StateStruct> tmp1 = new Stack<StateStruct>(ActionStack.Reverse());
                Stack<int> tmp2 = new Stack<int>(StateStack.Reverse());

                a.Invoke(this);
                while(ActionStack.Count > 0)
                {
                    var ak = ActionStack.Pop();
                    Parent.Transitions.Add(new TransitionStruct(ak.ch, ak.from, 0, n, ak.token));
                }
                
                // This is :(
                ActionStack = tmp1;
                StateStack = tmp2;
                return this;
            }

            public LexerRule ExitWhen(Action<LexerRule> a) => GotoWhen(null, a);

            public LexerRule Summon(string n)
            {
                Stack<StateStruct> tmp1 = new Stack<StateStruct>(ActionStack.Reverse());
                Stack<int> tmp2 = new Stack<int>(StateStack.Reverse());
                Parent.Actions[n].Invoke(this);
                ActionStack = tmp1;
                StateStack = tmp2;
                return this;
            }

            #endregion

            private int GetPreviousState() => StateStack.ElementAt(1);
            private int GetCurrentState() => StateStack.ElementAt(0);
            private int GetNextState() => Parent.TotalStates++;
        }
    }
}