using System.Collections.Generic;
using System.Threading.Tasks;
using JetBrains.Annotations;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace Cheetah.Matches.Realtime.Editor.UIElements.HistoryTextField
{
    /// <summary>
    /// Текстовое полей с поддержкой истории
    /// </summary>
    public class HistoryTextField : VisualElement
    {
        public class HistoryTextFieldUxmlTraits : UxmlTraits
        {
            private readonly UxmlStringAttributeDescription storageKey = new UxmlStringAttributeDescription { name = "storageKey" };

            public override void Init(VisualElement ve, IUxmlAttributes bag, CreationContext cc)
            {
                base.Init(ve, bag, cc);
                var element = ve as HistoryTextField;
                element.SetStorageKey(storageKey.GetValueFromBag(bag, cc));
            }
        }

        public new class UxmlFactory : UxmlFactory<HistoryTextField, HistoryTextFieldUxmlTraits>
        {
        }

        private void SetStorageKey(string storageKey)
        {
            this.storageKey = storageKey;
            history = new History(storageKey + "_history");
        }


        private History history;
        private bool firstUpdate = true;
        private string storageKey;
        private readonly TextField textField;

        public delegate Task ChangeListener(string value);

        private ChangeListener changeListener;

        public HistoryTextField()
        {
            var uiAsset =
                AssetDatabase.LoadAssetAtPath<VisualTreeAsset>("Packages/games.Cheetah.Matches.Realtime/Editor/UIElements/HistoryTextField/HistoryTextField.uxml");
            uiAsset.CloneTree(this);

            textField = this.Q<TextField>("textField");
            textField.isDelayed = false;
            textField.RegisterCallback<KeyDownEvent>(OnSearchFieldKeyPressed);
        }

        /// <summary>
        /// Колбек при нажатии клавиши во время фокуса на текстовом поле searchField. 
        /// Обратбатывает нажатия клавиш стрелка вверх, стрелка вниз и ввод(?). 
        /// </summary>
        /// <param name="evt"></param>
        private async void OnSearchFieldKeyPressed(KeyDownEvent evt)
        {
            switch (evt.keyCode)
            {
                case KeyCode.Return:
                case KeyCode.KeypadEnter:
                {
                    evt.PreventDefault();
                    EditorPrefs.SetString(storageKey, textField.value);
                    history.AddLine(textField.value);
                    textField.Focus();
                    await Apply(textField.value);
                }
                    break;

                case KeyCode.UpArrow:
                case KeyCode.DownArrow:
                    evt.PreventDefault();
                    if (evt.keyCode == KeyCode.UpArrow)
                    {
                        history.Up();
                    }
                    else
                    {
                        history.Down();
                    }

                    textField.value = history.Current();
                    textField.SelectAll();
                    break;
            }
        }


        public async Task Update()
        {
            if (firstUpdate)
            {
                firstUpdate = false;
                var current = EditorPrefs.GetString(storageKey, null);
                if (current != null)
                {
                    textField.value = current;
                    await Apply(textField.value);
                }
            }
        }

        private async Task Apply(string value)
        {
            if (changeListener != null)
            {
                await changeListener.Invoke(value);
            }
        }


        public void RegisterOnChangeListener(ChangeListener changeListener)
        {
            this.changeListener = changeListener;
        }
    }


    /// <summary>
    /// История введенных команд, сохраняется в EditorPrefs
    /// </summary>
    internal class History
    {
        private readonly HistoryStorage storage;
        private int index;
        private const int MAXHistoryItem = 10;
        private readonly string PrefsKey;

        public History(string PrefsKey)
        {
            this.PrefsKey = PrefsKey;
            var json = EditorPrefs.GetString(PrefsKey, "");
            storage = json != "" ? JsonConvert.DeserializeObject<HistoryStorage>(json) : new HistoryStorage();

            while (storage.lines.Count > MAXHistoryItem)
            {
                storage.lines.RemoveAt(0);
            }

            index = storage.lines.Count;
        }


        public void AddLine(string line)
        {
            if (storage.lines.Contains(line))
            {
                return;
            }

            storage.lines.Add(line);
            index = storage.lines.Count;

            EditorPrefs.SetString(PrefsKey, JsonConvert.SerializeObject(storage));
        }


        public void Up()
        {
            index++;
            if (index >= storage.lines.Count) index = storage.lines.Count - 1;
        }

        public void Down()
        {
            index--;
            if (index < 0) index = 0;
        }

        [CanBeNull]
        public string Current()
        {
            if (storage.lines.Count == 0)
            {
                return null;
            }
            return index < storage.lines.Count ? storage.lines[index] : null;
        }
    }

    /// <summary>
    /// Выделено в отдельных класс для использования Json
    /// </summary>
    public class HistoryStorage
    {
        public IList<string> lines = new List<string>();
    }
}