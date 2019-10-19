using System;
using System.Linq;
using System.Collections.Generic;

namespace spoodly
{
    public enum LexerState 
    {
        Default,
        
        String_Body,
        String_Escape,

        Integer_Body,
        Integer_HexBody,
        Integer_BinaryBody,
        Integer_Prefix,
        Integer_Decimal,
        Integer_Numeric,
        Integer_SignedNumeric
    }

    public enum LexerToken
    {
        Unknown,

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

        // Char Literals
        CharLiteral
    }

    public class Lexer
    {
        private Dictionary<(char c, LexerState state), (LexerState state, LexerToken token)> Lut;

        // TODO: I don't really like building the grammar like this... Maybe use Linq?
        public Lexer()
        {
            // Initialize data structures
            Lut = new Dictionary<(char c, LexerState state), (LexerState state, LexerToken token)>();
            var chars = Enumerable.Range(0, char.MaxValue+1).Select(i => (char) i).Where(c => !char.IsControl(c)).ToArray();

            // Set default action for all characters
            Lut[((char) 0xFF, LexerState.Default)] = (LexerState.Default, LexerToken.Unknown);

            // Build string literal parsing DFA
            // Find opening and closing quotes, ignoring escape sequences '\' - we don't support '\x' yet
            Lut[('"',   LexerState.Default)]     = (LexerState.String_Body,   LexerToken.StringLiteral);
            Lut[('"',   LexerState.String_Body)] = (LexerState.Default,       LexerToken.StringLiteral);
            Lut[('\\',  LexerState.String_Body)] = (LexerState.String_Escape, LexerToken.StringLiteral);
            Lut[((char) 0xFF, LexerState.String_Escape)] = (LexerState.String_Body, LexerToken.StringLiteral);
            Lut[((char) 0xFF, LexerState.String_Body)]   = (LexerState.String_Body, LexerToken.StringLiteral);

            // Build numeric literal parsing DFA
            // First parse '0' prefix
            Lut[('0', LexerState.Default)] = (LexerState.Integer_Prefix, LexerToken.IntegerLiteral);
            "123456789".ToList().ForEach((c) => Lut[(c, LexerState.Default)] = (LexerState.Integer_Body, LexerToken.IntegerLiteral));
            // Find out whether it's binary or hex
            Lut[('x', LexerState.Integer_Prefix)] = (LexerState.Integer_HexBody, LexerToken.IntegerHexLiteral);
            Lut[('X', LexerState.Integer_Prefix)] = (LexerState.Integer_HexBody, LexerToken.IntegerHexLiteral);
            Lut[('b', LexerState.Integer_Prefix)] = (LexerState.Integer_BinaryBody, LexerToken.IntegerBinaryLiteral);
            Lut[('B', LexerState.Integer_Prefix)] = (LexerState.Integer_BinaryBody, LexerToken.IntegerBinaryLiteral);
            // If it's none of the prefixes stated above, break and treat as regular integer
            "0123456789".ToList().ForEach((c) => Lut[(c, LexerState.Integer_Prefix)] = (LexerState.Integer_Body, LexerToken.IntegerLiteral));
            "0123456789".ToList().ForEach((c) => Lut[(c, LexerState.Integer_Body)] = (LexerState.Integer_Body, LexerToken.IntegerLiteral));
            // Otherwise, parse the integer
            "0123456789ABCDEFabcdef".ToList().ForEach((c) => Lut[(c, LexerState.Integer_HexBody)] = (LexerState.Integer_HexBody, LexerToken.IntegerHexLiteral));
            Lut[('0', LexerState.Integer_BinaryBody)] = (LexerState.Integer_BinaryBody, LexerToken.IntegerBinaryLiteral);
            Lut[('1', LexerState.Integer_BinaryBody)] = (LexerState.Integer_BinaryBody, LexerToken.IntegerBinaryLiteral);
            // If it's a regular integer, try looking for floating point notation
            // After decimal points, we'll accept numberic and e
            Lut[('.', LexerState.Integer_Body)] = (LexerState.Integer_Decimal, LexerToken.DecimalPoint);
            "0123456789".ToList().ForEach((c) => Lut[(c, LexerState.Integer_Decimal)] = (LexerState.Integer_Decimal, LexerToken.IntegerLiteral));
            // After e we'll accept only numberic values
            Lut[('e', LexerState.Integer_Body)] = (LexerState.Integer_SignedNumeric, LexerToken.ExponentIdentifier);
            Lut[('E', LexerState.Integer_Body)] = (LexerState.Integer_SignedNumeric, LexerToken.ExponentIdentifier);
            Lut[('e', LexerState.Integer_Decimal)] = (LexerState.Integer_SignedNumeric, LexerToken.ExponentIdentifier);
            Lut[('E', LexerState.Integer_Decimal)] = (LexerState.Integer_SignedNumeric, LexerToken.ExponentIdentifier);
            // After sign, we only accept numeric values
            "+-".ToList().ForEach((c) => Lut[(c, LexerState.Integer_SignedNumeric)] = (LexerState.Integer_Numeric, LexerToken.ExponentSign));
            // If no sign is specified, we only accept numeric values
            "0123456789".ToList().ForEach((c) => Lut[(c, LexerState.Integer_SignedNumeric)] = (LexerState.Integer_Numeric, LexerToken.IntegerLiteral));
            // Numeric values only now
            "0123456789".ToList().ForEach((c) => Lut[(c, LexerState.Integer_Numeric)] = (LexerState.Integer_Numeric, LexerToken.IntegerLiteral));
            // Reset DFA states now..., we will not allow reset after "e", "e+/-", ".", "0x", or "0b" specifiers

        }

        public LexerToken[] Parse(string text)
        {
            LexerState currentState = LexerState.Default;
            LexerToken[] states = new LexerToken[text.Length];
            for(int i = 0; i < text.Length; i++)
            {
                (LexerState state, LexerToken token) res;
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

    // TODO: Idk what to do with this >.<
    public class LexerException : Exception
    {
        public LexerException()
        {
        }

        public LexerException(string message)
            : base(message)
        {
        }

        public LexerException(string message, Exception inner)
            : base(message, inner)
        {
        }
    }
}