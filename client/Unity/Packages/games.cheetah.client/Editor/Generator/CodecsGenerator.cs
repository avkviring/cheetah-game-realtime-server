using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using Games.Cheetah.Client.Codec;
using UnityEditor;
using UnityEditor.Compilation;
using UnityEngine;
using Assembly = UnityEditor.Compilation.Assembly;

namespace Games.Cheetah.Client.Editor.Generator
{
    public static class CodecsGenerator
    {
        [MenuItem("Window/Cheetah/Generate codecs", priority = 100)]
        public static void Generate()
        {
            var locationByName = CompilationPipeline.GetAssemblies()
                .Where(assembly => assembly.sourceFiles.Length != 0)
                .ToDictionary(assembly => assembly.name);

            var formatters = new Formatters();
            var assemblies = AppDomain.CurrentDomain.GetAssemblies();
            foreach (var assembly in assemblies)
            {
                if (!locationByName.TryGetValue(assembly.GetName().Name, out var unityAssembly))
                {
                    continue;
                }

                foreach (var type in assembly.GetTypes())
                {
                    var generateCodecAttribute = type.GetCustomAttribute<GenerateCodec>();
                    if (generateCodecAttribute == null) continue;
                    string rootNamespace = null;
                    if (type.Namespace is not null)
                    {
                        rootNamespace = type.Namespace.Replace(".", "_");
                    }
                    var result = new CodecGenerator(formatters, rootNamespace, type).Generate();


                    var assemblyDirectory = IsDefaultAssembly(unityAssembly)
                        ? GetDefaultAssemblyRootPath()
                        : Path.GetDirectoryName(CompilationPipeline.GetAssemblyDefinitionFilePathFromAssemblyName(unityAssembly.name));
                    var generatedCodecDirectory = assemblyDirectory + "/_Generated/";
                    Directory.CreateDirectory(generatedCodecDirectory);

                    var generatedCodecFile = Utils.GetFullName(type) + "Codec.cs";
                    File.WriteAllBytes(generatedCodecDirectory + "/" + generatedCodecFile, Encoding.UTF8.GetBytes(result));
                }
            }

            CompilationPipeline.RequestScriptCompilation();
        }

        private static string GetDefaultAssemblyRootPath()
        {
            return Application.dataPath + "/Scripts/";
        }

        private static bool IsDefaultAssembly(Assembly unityAssembly)
        {
            return unityAssembly.name == "Assembly-CSharp";
        }
    }
}