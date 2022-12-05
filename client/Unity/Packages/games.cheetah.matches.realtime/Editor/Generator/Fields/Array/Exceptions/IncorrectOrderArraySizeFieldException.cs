using System;
using Cheetah.Matches.Realtime.Codec;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields.Array.Exceptions
{
    public class IncorrectOrderArraySizeFieldException : Exception
    {
        public IncorrectOrderArraySizeFieldException(string field) : base(
            $"Field {field} should defined before when field with {nameof(ArraySizeField)}.")
        {
        }
    }
}