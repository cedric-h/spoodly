using System;
using System.Collections.Generic;

namespace spoodly
{
    public enum LexerToken
    {
        Unknown = 0,
        // String Literals
        StringLiteral,
        // Integer Literals
        IntegerLiteral,
        IntegerHexLiteral,
        IntegerBinaryLiteral,
        DecimalPoint,
        ExponentIdentifier,
        HexSeparator,
        BinarySeparator,
        ExponentSign,
        // Open parenthesis
        OpenParenthesis,
        CloseParenthesis,
        // Whitespace!
        Whitespace,
        // Operators and whatnot
        Operator,
        Comparator,
        VerboseOperator,
        VerboseComparator,
        // Identifiers
        Identifier,
    }

    public class DynamicLexer
    {
        public Dictionary<(char c, int state), (int state, LexerToken token)> Lut;
        public DynamicLexer()
        {
            this.Lut = new Dictionary<(char c, int state), (int state, LexerToken token)>();

            // Create the grammar Lut now
            var myGrammar = new LexerGrammar<LexerToken>(Lut);

            // Add default case for any char
            myGrammar.AddNewRule().MatchAny(LexerRule.AnyChar, LexerToken.Unknown);
            myGrammar.AddNewRule().MatchAny(' ', LexerToken.Whitespace);

            // Add parsing for identifiers
            myGrammar.AddNewRule().MatchAny('(', LexerToken.OpenParenthesis);
            myGrammar.AddNewRule().MatchAny(')', LexerToken.CloseParenthesis);

            // Add parsing for verbose operators/comparators
            myGrammar.AddNewRule()
                .MatchString("AND ", LexerToken.Unknown, LexerToken.VerboseComparator)
                .MatchString("OR " , LexerToken.Unknown, LexerToken.VerboseComparator)
                .MatchString("NOT ", LexerToken.Unknown, LexerToken.VerboseComparator)
                .MatchString("EQUALS ", LexerToken.Unknown, LexerToken.VerboseComparator)
                .MatchString("MOD ", LexerToken.Unknown, LexerToken.VerboseOperator);
            
            // Add parsing for symbolic operators/comparators
            myGrammar.AddNewAction("Operators",
            t => t
                .MatchString("<-", LexerToken.Unknown, LexerToken.Operator)
                .MatchString("+", LexerToken.Unknown, LexerToken.Operator)
                .MatchString("-", LexerToken.Unknown, LexerToken.Operator)
                .MatchString("*", LexerToken.Unknown, LexerToken.Operator)
                .MatchString("/", LexerToken.Unknown, LexerToken.Operator)
                .MatchString(">=", LexerToken.Unknown, LexerToken.Comparator)
                .MatchString("<=", LexerToken.Unknown, LexerToken.Comparator)
                .MatchString(">", LexerToken.Unknown, LexerToken.Comparator)
                .MatchString("<", LexerToken.Unknown, LexerToken.Comparator)
                .MatchString("=", LexerToken.Unknown, LexerToken.Comparator)
            );
            // Summon the devil!
            myGrammar.AddNewRule().Summon("Operators");

            // Add parsing for string literals
            myGrammar.AddNewRule().Match('"', LexerToken.StringLiteral).Then(
            t1 => t1
                .MatchAny(LexerRule.AnyChar, LexerToken.StringLiteral)
                .Until('"', LexerToken.StringLiteral)
                .Match('\\', LexerToken.StringLiteral).Then(
                t2 => t2
                    .Until(LexerRule.AnyChar, LexerToken.StringLiteral)
                )
            );
            
            // Add parsing for numeric literals
            myGrammar.AddNewAction("NumericExitAction", t => t.ExitWhen(
            t1 => t1
                .Match(' ', LexerToken.Whitespace)
                .Match(')', LexerToken.CloseParenthesis)
            ).Summon("Operators"));

            myGrammar.AddNewRule().Match('0', LexerToken.IntegerLiteral).Then(
            t1 => t1
                .Match("xX", LexerToken.HexSeparator).Then(
                t2 => t2
                    .MatchAny("0123456789ABCDEFabcdef", LexerToken.IntegerHexLiteral)
                    .Summon("NumericExitAction")
                ).Match("bB", LexerToken.BinarySeparator).Then(
                t2 => t2
                    .MatchAny("01", LexerToken.IntegerBinaryLiteral)
                    .Summon("NumericExitAction")
                ).GotoWhen("Integers", t2 => t2.Match(LexerRule.Numerics, LexerToken.IntegerLiteral))
                 .GotoWhen("Decimals", t2 => t2.Match('.', LexerToken.DecimalPoint))
                 .Summon("NumericExitAction")
            ).Match("123456789", LexerToken.IntegerLiteral).Then(
            t1 => t1
                .Name("Integers").MatchAny(LexerRule.Numerics, LexerToken.IntegerLiteral)
                .Match('.', LexerToken.DecimalPoint)
                .Summon("NumericExitAction").Then(
                t2 => t2
                    .Name("Decimals").MatchAny(LexerRule.Numerics, LexerToken.IntegerLiteral)
                    .Summon("NumericExitAction")
                    .GotoWhen("Exponents", t3 => t3.Match("Ee", LexerToken.ExponentIdentifier))
                ).Match("Ee", LexerToken.ExponentIdentifier).Then(
                t2 => t2
                    .Name("Exponents").Match("+-", LexerToken.ExponentSign).Then(
                    t3 => t3
                        .MatchAny(LexerRule.Numerics, LexerToken.IntegerLiteral)
                        .Summon("NumericExitAction")
                    ).Match(LexerRule.Numerics, LexerToken.IntegerLiteral).Then(t3 => t3
                        .MatchAny(LexerRule.Numerics, LexerToken.IntegerLiteral)
                        .Summon("NumericExitAction")
                    )
                )
            );

            myGrammar.ConstructLut();
            Console.WriteLine($"Constructed LUT. Total states: {myGrammar.TotalStates}");
        }

        public LexerToken[] Parse(string text)
        {
            int currentState = 0;
            LexerToken[] states = new LexerToken[text.Length];
            for (int i = 0; i < text.Length; i++)
            {
                (int state, LexerToken token) res;
                // We take advantage of C# operator short circuiting
                if (Lut.TryGetValue((text[i], currentState), out res) || Lut.TryGetValue(((char)0xFF, currentState), out res))
                {
                    currentState = res.state;
                    states[i] = res.token;
                }
                else
                    throw new UnknownLexerStateException(i, text[i], currentState);
            }
            return states;
        }
    }

    public class UnknownLexerStateException : Exception
    {
        private string _internalMessage;
        private int col;
        private char c;
        private int state;

        public override string Message
        {
            get => $"Lexer was put in an undefined state: (Column: {col}, Character: '{c}', State: {state})";
        }

        public UnknownLexerStateException(int col, char c, int state)
        {
            this.col = col;
            this.c = c;
            this.state = state;
        }

        public UnknownLexerStateException(string message, int col, char c, int state)
        {
            this._internalMessage = message;
            this.col = col;
            this.c = c;
            this.state = state;
        }
    }
}