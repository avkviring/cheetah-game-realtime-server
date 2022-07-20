using System;
using Cheetah.Matches.Relay.Codec;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions
{
    public class IncorrectOrderArraySizeFieldException : Exception
    {
        public IncorrectOrderArraySizeFieldException(string field) : base(
            $"Field {field} should defined before when field with {nameof(ArraySizeField)}.")
        {
        }
    }
}