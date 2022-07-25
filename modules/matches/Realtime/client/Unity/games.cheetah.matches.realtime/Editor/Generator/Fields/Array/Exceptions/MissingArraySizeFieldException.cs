using System;
using Cheetah.Matches.Realtime.Codec;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields.Array.Exceptions
{
    public class MissingArraySizeFieldException : Exception
    {
        public MissingArraySizeFieldException(string field) : base($"Field {field} set in {nameof(ArraySizeField)} but not present in structure.")
        {
        }
    }
}