using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using UnityEngine;
using Games.Cheetah.Client.Tests.Codec.Field;

// ReSharper disable once CheckNamespace
namespace Games_Cheetah_Client_Tests_Codec_Field
{
		// warning warning warning warning warning
		// Code generated by Cheetah relay codec generator - DO NOT EDIT
		// warning warning warning warning warning
		public class TestCodecFieldInnerCodec:Codec<Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Inner>
		{
			public void Decode(ref NetworkBuffer buffer, ref Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Inner dest)
			{
				dest.value = IntFormatter.Instance.Read(ref buffer);
			}
	
			public void  Encode(in Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Inner source, ref NetworkBuffer buffer)
			{
				IntFormatter.Instance.Write(source.value,ref buffer);
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			private static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new TestCodecFieldInnerCodec());
			}
	
		}
	
	
}
